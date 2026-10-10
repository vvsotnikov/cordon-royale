mod game;
mod progression;
mod world;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use std::{
    collections::BTreeMap,
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
async fn upgrade(
    ws: WebSocketUpgrade,
    Query(query): Query<BTreeMap<String, String>>,
    State(app): State<App>,
) -> impl IntoResponse {
    let token = query.get("token").cloned().unwrap_or_default();
    ws.max_message_size(2048)
        .max_frame_size(2048)
        .on_upgrade(move |socket| connection(socket, app, token))
}
async fn connection(mut socket: WebSocket, app: App, token: String) {
    if token.len() != 64 || !token.bytes().all(|c| c.is_ascii_hexdigit()) {
        let _ = socket
            .send(Message::Text(
                "{\"kind\":\"error\",\"text\":\"Invalid player identity. Refresh to retry.\"}"
                    .into(),
            ))
            .await;
        return;
    }
    let id = app.ids.fetch_add(1, Ordering::Relaxed);
    if !app.game.lock().await.join(id, token) {
        let _=socket.send(Message::Text("{\"kind\":\"error\",\"text\":\"Room full or this profile is already connected in another tab.\"}".into())).await;
        return;
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
        app.game.lock().await.leave(id);
        return;
    }
    let mut window = std::time::Instant::now();
    let mut count = 0;
    loop {
        tokio::select! {
         value=rx.recv()=>match value {Ok(s)=>if send.send(Message::Text(s.into())).await.is_err(){break},Err(broadcast::error::RecvError::Lagged(_))=>continue,Err(_)=>break},
         message=recv.next()=>match message {
          Some(Ok(Message::Text(s)))=>{
           if window.elapsed().as_secs()>=1{window=std::time::Instant::now();count=0;}count+=1;if count>100{break;}
           if let Ok(v)=serde_json::from_str(&s){app.game.lock().await.command(id,v);}
          },
          Some(Ok(Message::Ping(p)))=>if send.send(Message::Pong(p)).await.is_err(){break},
          Some(Ok(Message::Close(_)))|None|Some(Err(_))=>break,_=>{}
         }
        }
    }
    app.game.lock().await.leave(id);
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let assets = PathBuf::from(
        std::env::var("CORDON_ASSETS").unwrap_or("D:/GameAssets/cordon-royale/web".into()),
    );
    let saves = PathBuf::from(
        std::env::var("CORDON_SAVES").unwrap_or("D:/GameAssets/cordon-royale/profiles.json".into()),
    );
    let profiles = match tokio::fs::read(&saves).await {
        Ok(bytes) => serde_json::from_slice(&bytes)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
        Err(e) => return Err(e.into()),
    };
    println!("Loading local collision mesh...");
    let world = world::World::load(&assets.join("collision.cform"))?;
    let (tx, _) = broadcast::channel(8);
    let app = App {
        game: Arc::new(Mutex::new(game::Game::new(world, profiles))),
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
            let _ = sim.tx.send(g.snapshot().to_string());
        }
    });
    let persistence = app.clone();
    tokio::spawn(async move {
        let mut timer = tokio::time::interval(std::time::Duration::from_secs(2));
        loop {
            timer.tick().await;
            let bytes = {
                let mut g = persistence.game.lock().await;
                if !g.dirty {
                    continue;
                }
                g.dirty = false;
                serde_json::to_vec(&g.profiles).expect("serializable profiles")
            };
            let result = async {
                if let Some(parent) = saves.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                let temp = saves.with_extension("json.tmp");
                tokio::fs::write(&temp, &bytes).await?;
                tokio::fs::rename(&temp, &saves).await
            }
            .await;
            if let Err(e) = result {
                eprintln!("Profile save failed: {e}");
                persistence.game.lock().await.dirty = true;
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
    println!("Cordon Evolution: http://{address}");
    axum::serve(listener, router).await?;
    Ok(())
}
