mod game;
mod world;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use std::{
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::sync::{broadcast, Mutex};
use tower_http::services::ServeDir;
#[derive(Clone)]
struct App {
    game: Arc<Mutex<game::Game>>,
    tx: broadcast::Sender<String>,
    ids: Arc<AtomicU64>,
}
async fn upgrade(ws: WebSocketUpgrade, State(app): State<App>) -> impl IntoResponse {
    ws.max_message_size(2048)
        .max_frame_size(2048)
        .on_upgrade(move |socket| connection(socket, app))
}
async fn connection(socket: WebSocket, app: App) {
    let id = app.ids.fetch_add(1, Ordering::Relaxed);
    {
        let mut g = app.game.lock().await;
        if g.players.values().filter(|p| !p.bot).count() >= 12 {
            return;
        }
        g.join(id, false);
    }
    let (mut send, mut recv) = socket.split();
    let mut rx = app.tx.subscribe();
    if send
        .send(Message::Text(
            serde_json::json!({"kind":"welcome","id":id})
                .to_string()
                .into(),
        ))
        .await
        .is_err()
    {
        app.game.lock().await.players.remove(&id);
        return;
    }
    loop {
        tokio::select! {value=rx.recv()=>{match value{Ok(s)=>{if send.send(Message::Text(s.into())).await.is_err(){break}},Err(broadcast::error::RecvError::Lagged(_))=>continue,Err(_)=>break}},message=recv.next()=>{match message{Some(Ok(Message::Text(s)))=>{let Ok(v)=serde_json::from_str::<serde_json::Value>(&s)else{continue};let mut g=app.game.lock().await;let time=g.time;if let Some(p)=g.players.get_mut(&id){if v["kind"]=="name"{p.name=v["name"].as_str().unwrap_or("Stalker").chars().filter(|c|c.is_alphanumeric()||*c==' '||*c=='_'||*c=='-').take(20).collect();if p.name.is_empty(){p.name="Stalker".into()}}else if let Ok(input)=serde_json::from_value::<game::Input>(v){if input.valid(){p.input=input;p.last_input=time;}}}},Some(Ok(Message::Ping(p)))=>{if send.send(Message::Pong(p)).await.is_err(){break}},Some(Ok(Message::Close(_)))|None|Some(Err(_))=>break,_=>{}}}}
    }
    app.game.lock().await.players.remove(&id);
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let assets = PathBuf::from(
        std::env::var("CORDON_ASSETS").unwrap_or("D:/GameAssets/cordon-royale/web".into()),
    );
    println!("Loading local collision mesh...");
    let world = world::World::load(&assets.join("collision.cform"))?;
    let (tx, _) = broadcast::channel(8);
    let app = App {
        game: Arc::new(Mutex::new(game::Game::new(world))),
        tx,
        ids: Arc::new(AtomicU64::new(10000)),
    };
    let sim = app.clone();
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(std::time::Duration::from_millis(50));
        timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            timer.tick().await;
            let mut g = sim.game.lock().await;
            g.tick();
            if let Ok(s) = serde_json::to_string(&g.snapshot()) {
                let _ = sim.tx.send(s);
            }
        }
    });
    let router = Router::new()
        .route("/ws", get(upgrade))
        .route("/health", get(|| async { "ok" }))
        .nest_service("/assets", ServeDir::new(assets))
        .fallback_service(ServeDir::new("public"))
        .with_state(app);
    let address = std::env::var("BIND").unwrap_or("127.0.0.1:8787".into());
    let listener = tokio::net::TcpListener::bind(&address).await?;
    println!("Cordon Royale: http://{address}");
    axum::serve(listener, router).await?;
    Ok(())
}
