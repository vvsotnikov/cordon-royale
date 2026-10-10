use crate::{progression::*, world::*};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
#[derive(Clone, Default, Deserialize)]
#[serde(default)]
pub struct Input {
    pub forward: f32,
    pub right: f32,
    pub yaw: f32,
    pub sprint: bool,
}
impl Input {
    pub fn valid(&self) -> bool {
        [self.forward, self.right, self.yaw]
            .iter()
            .all(|v| v.is_finite())
            && self.forward.abs() <= 1.
            && self.right.abs() <= 1.
    }
}
#[derive(Serialize)]
pub struct Player {
    pub id: u64,
    pub pos: V,
    pub yaw: f32,
    pub profile: Profile,
    pub duel: Option<usize>,
    pub duel_remaining: f32,
    pub training: f32,
    #[serde(skip)]
    pub token: String,
    #[serde(skip)]
    pub input: Input,
    #[serde(skip)]
    pub last_input: f64,
    #[serde(skip)]
    pub next_train: f64,
}
#[derive(Serialize)]
pub struct Opponent {
    pub name: &'static str,
    pub title: &'static str,
    pub pos: V,
    pub power: u64,
    pub wins: u64,
    pub rubles: u64,
    pub species: &'static str,
    pub drop: Option<usize>,
}
#[derive(Serialize)]
pub struct Event {
    pub seq: u64,
    pub who: u64,
    pub kind: String,
    pub text: String,
}
pub struct Game {
    pub world: World,
    pub players: BTreeMap<u64, Player>,
    pub profiles: BTreeMap<String, Profile>,
    pub opponents: Vec<Opponent>,
    pub events: Vec<Event>,
    pub time: f64,
    seq: u64,
    pub dirty: bool,
}
impl Game {
    pub fn new(world: World, mut profiles: BTreeMap<String, Profile>) -> Self {
        for profile in profiles.values_mut() {
            profile.sanitize();
        }
        let definitions = [
            ("Blind dog", "Mutant", -202., 153., 20, 1, 15, "dog", None),
            (
                "Bandit",
                "Rookie raider",
                -194.,
                157.,
                100,
                3,
                50,
                "bandit",
                Some(1),
            ),
            (
                "Bloodsucker",
                "Mutant",
                -210.,
                162.,
                500,
                8,
                180,
                "bloodsucker",
                Some(6),
            ),
            (
                "Mercenary",
                "Professional killer",
                -226.,
                155.,
                2500,
                20,
                600,
                "mercenary",
                Some(3),
            ),
            (
                "Controller",
                "Mutant",
                -231.,
                143.,
                12000,
                50,
                2000,
                "controller",
                Some(4),
            ),
        ];
        let opponents = definitions
            .into_iter()
            .map(
                |(name, title, x, z, power, wins, rubles, species, drop)| Opponent {
                    name,
                    title,
                    pos: [x, world.floor(x, z, -15.).unwrap_or(-20.), z],
                    power,
                    wins,
                    rubles,
                    species,
                    drop,
                },
            )
            .collect();
        Self {
            world,
            profiles,
            players: BTreeMap::new(),
            opponents,
            events: vec![],
            time: 0.,
            seq: 0,
            dirty: false,
        }
    }
    pub fn join(&mut self, id: u64, token: String) -> bool {
        if self.players.len() >= 12 || self.players.values().any(|p| p.token == token) {
            return false;
        }
        let mut profile = self.profiles.get(&token).cloned().unwrap_or_default();
        profile.sanitize();
        let x = -205.;
        let z = 150.;
        let y = self.world.floor(x, z, -15.).unwrap_or(-20.);
        self.players.insert(
            id,
            Player {
                id,
                token,
                profile,
                pos: [x, y, z],
                yaw: 0.,
                duel: None,
                duel_remaining: 0.,
                training: 0.,
                input: Input::default(),
                last_input: self.time,
                next_train: 0.,
            },
        );
        true
    }
    pub fn remember(&mut self, id: u64) {
        if let Some(p) = self.players.get(&id) {
            self.profiles.insert(p.token.clone(), p.profile.clone());
            self.dirty = true;
        }
    }
    pub fn leave(&mut self, id: u64) {
        self.remember(id);
        self.players.remove(&id);
    }
    fn event(&mut self, id: u64, kind: &str, text: String) {
        self.seq += 1;
        self.events.push(Event {
            seq: self.seq,
            who: id,
            kind: kind.into(),
            text,
        });
        if self.events.len() > 48 {
            self.events.remove(0);
        }
    }
    pub fn command(&mut self, id: u64, v: serde_json::Value) {
        let Some(p) = self.players.get_mut(&id) else {
            return;
        };
        let kind = v["kind"].as_str().unwrap_or("");
        let message = match kind {
            "name" => {
                p.profile.name = v["name"].as_str().unwrap_or("Stalker").into();
                p.profile.sanitize();
                None
            }
            "train" if p.duel.is_none() && self.time >= p.next_train => {
                p.next_train = self.time + 0.19;
                p.training = 0.24;
                p.profile.train();
                None
            }
            "buy" | "equip" if p.duel.is_none() => {
                let n = v["item"]
                    .as_u64()
                    .and_then(|n| usize::try_from(n).ok())
                    .unwrap_or(usize::MAX);
                Some(
                    match if kind == "buy" {
                        p.profile.buy(n)
                    } else {
                        p.profile.equip(n)
                    } {
                        Ok(()) => format!("Equipped {}", ITEMS[n].name),
                        Err(e) => e.into(),
                    },
                )
            }
            "promote" if p.duel.is_none() => Some(match p.profile.promote() {
                Ok(()) => format!(
                    "Promoted to {}! Training multiplier ×{}",
                    RANKS[p.profile.rank],
                    1_u64 << p.profile.rank
                ),
                Err(e) => e.into(),
            }),
            "home" if p.duel.is_none() => {
                p.pos = [
                    -205.,
                    self.world.floor(-205., 150., -15.).unwrap_or(-20.),
                    150.,
                ];
                p.input = Input::default();
                Some("Returned to camp.".into())
            }
            "challenge" if p.duel.is_none() => {
                if let Some(n) = v["opponent"]
                    .as_u64()
                    .and_then(|n| usize::try_from(n).ok())
                    .filter(|&n| n < self.opponents.len())
                {
                    let opponent = &self.opponents[n];
                    if dist(p.pos, opponent.pos) > 6. {
                        Some(format!(
                            "Walk closer to {} to challenge them.",
                            opponent.name
                        ))
                    } else {
                        p.duel = Some(n);
                        p.duel_remaining = 3.;
                        Some(format!("Fighting {}…", opponent.name))
                    }
                } else {
                    None
                }
            }
            "input" => {
                if let Ok(input) = serde_json::from_value::<Input>(v) {
                    if input.valid() {
                        p.input = input;
                        p.last_input = self.time;
                    }
                }
                None
            }
            _ => return,
        };
        self.remember(id);
        if let Some(text) = message {
            self.event(id, "notice", text);
        }
    }
    pub fn tick(&mut self) {
        self.time += 0.05;
        let mut finished = vec![];
        for (&id, p) in &mut self.players {
            p.training = (p.training - 0.05).max(0.);
            if self.time - p.last_input > 0.3 {
                p.input = Input::default();
            }
            if let Some(n) = p.duel {
                p.duel_remaining = (p.duel_remaining - 0.05).max(0.);
                if p.duel_remaining == 0. {
                    let o = &self.opponents[n];
                    let won = p.profile.strength() >= o.power;
                    let loot = if won {
                        p.profile.reward(n, o.wins, o.rubles, o.drop)
                    } else {
                        None
                    };
                    finished.push((
                        id,
                        won,
                        if won {
                            format!(
                                "Defeated {} · +{} wins · +{} rubles{}",
                                o.name,
                                o.wins,
                                o.rubles,
                                loot.map(|item| format!(" · Loot: {item}"))
                                    .unwrap_or_default()
                            )
                        } else {
                            format!(
                                "{} won. Reach {} combat strength and try again.",
                                o.name, o.power
                            )
                        },
                    ));
                    p.duel = None;
                }
                continue;
            }
            p.yaw = p.input.yaw;
            let f = p.input.forward;
            let r = p.input.right;
            let scale =
                if p.input.sprint { 7.4 } else { 4.8 } * 0.05 / (f * f + r * r).sqrt().max(1.);
            p.pos = self.world.move_player(
                p.pos,
                (-p.yaw.sin() * f + p.yaw.cos() * r) * scale,
                (-p.yaw.cos() * f - p.yaw.sin() * r) * scale,
            );
        }
        for (id, won, text) in finished {
            self.remember(id);
            self.event(id, if won { "win" } else { "loss" }, text);
        }
    }
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({"kind":"state","time":self.time,"players":self.players.values().collect::<Vec<_>>(),"opponents":self.opponents,"events":self.events,"items":ITEMS,"ranks":RANKS,"promotions":PROMOTIONS})
    }
}
