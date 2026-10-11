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
    pub jump: bool,
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
    pub duel_progress: f64,
    pub attack_seq: u64,
    pub attack_target: Option<usize>,
    #[serde(skip)]
    pub next_attack: f64,
    pub sitting: bool,
    #[serde(skip)]
    pub token: String,
    #[serde(skip)]
    pub input: Input,
    #[serde(skip)]
    pub last_input: f64,
    #[serde(skip)]
    pub next_train: f64,
    #[serde(skip)]
    pub vertical_velocity: f32,
    #[serde(skip)]
    pub jump_held: bool,
    #[serde(skip)]
    pub jump_origin: V,
}
#[derive(Serialize)]
pub struct Opponent {
    pub name: &'static str,
    pub title: &'static str,
    pub pos: V,
    pub power: u64,
    pub exp: u64,
    pub reward_pos: V,
    pub species: &'static str,
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
            ("Blind dog", "Mutant", -202., 153., 20, 1, "dog"),
            ("Bandit", "Raider", -194., 157., 100, 5, "bandit"),
            ("Bloodsucker", "Mutant", -210., 162., 500, 25, "bloodsucker"),
            ("Mercenary", "Killer", -226., 155., 2500, 100, "mercenary"),
            (
                "Controller",
                "Mutant",
                -231.,
                143.,
                12000,
                500,
                "controller",
            ),
        ];
        let opponents = definitions
            .into_iter()
            .map(|(name, title, x, z, power, exp, species)| Opponent {
                name,
                title,
                pos: [x, world.floor(x, z, -15.).unwrap_or(-20.), z],
                power,
                exp,
                reward_pos: [x + 2., world.floor(x + 2., z, -15.).unwrap_or(-20.), z],
                species,
            })
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
    fn campfire(&self) -> V {
        [-206.817, -19.97, 140.435]
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
                duel_progress: 0.5,
                attack_seq: 0,
                attack_target: None,
                next_attack: 0.,
                sitting: false,
                input: Input::default(),
                last_input: self.time,
                next_train: 0.,
                vertical_velocity: 0.,
                jump_held: false,
                jump_origin: [x, y, z],
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
        let campfire = self.campfire();
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
            "sit" if p.duel.is_none() && p.vertical_velocity == 0. => {
                if dist(p.pos, campfire) > 3.5 {
                    Some("Walk to the campfire to sit.".into())
                } else {
                    p.sitting = !p.sitting;
                    p.next_train = self.time + 1.;
                    p.input = Input::default();
                    None
                }
            }
            "equip" if p.duel.is_none() => {
                let n = v["item"].as_u64().unwrap_or(u64::MAX) as usize;
                Some(match p.profile.equip(n) {
                    Ok(()) => format!("Equipped {}", ITEMS[n].name),
                    Err(e) => e.into(),
                })
            }
            "ascend" if p.duel.is_none() => {
                if v["tier"].as_u64() != Some((p.profile.ascension + 1) as u64) {
                    return;
                }
                Some(match p.profile.ascend() {
                    Ok(()) => format!(
                        "Ascended: {} · +{} power / second",
                        ARMORS[p.profile.ascension].name,
                        p.profile.gain()
                    ),
                    Err(e) => e.into(),
                })
            }
            "open_box" if p.duel.is_none() => Some(match p.profile.open_box(random_roll()) {
                Ok(n) => format!("Lootbox: {n}"),
                Err(e) => e.into(),
            }),
            "claim_exp" if p.duel.is_none() && p.vertical_velocity == 0. => {
                if let Some(n) = v["stage"]
                    .as_u64()
                    .and_then(|n| usize::try_from(n).ok())
                    .filter(|&n| n < self.opponents.len())
                {
                    let o = &self.opponents[n];
                    let marker = o.reward_pos;
                    if !p.profile.reward_unlocked(n) {
                        Some("Win this run: defeat the dog, then the preceding stages.".into())
                    } else if dist(p.pos, marker) > 2.5 {
                        Some("Walk to the EXP marker beside this enemy.".into())
                    } else {
                        let reward = if n == 4 && p.profile.run_wins.len() == 5 {
                            2500
                        } else {
                            o.exp
                        };
                        p.profile.exp = p.profile.exp.saturating_add(reward);
                        p.profile.run_wins.clear();
                        p.pos = [
                            -205.,
                            self.world.floor(-205., 150., -15.).unwrap_or(-20.),
                            150.,
                        ];
                        p.input = Input::default();
                        p.sitting = false;
                        Some(format!("+{} EXP · Returned to camp · New run", reward))
                    }
                } else {
                    None
                }
            }
            "home" if p.duel.is_none() => {
                p.pos = [
                    -205.,
                    self.world.floor(-205., 150., -15.).unwrap_or(-20.),
                    150.,
                ];
                p.sitting = false;
                p.input = Input::default();
                p.vertical_velocity = 0.;
                p.jump_held = false;
                Some("Returned to camp.".into())
            }
            "challenge" if p.duel.is_none() && p.vertical_velocity == 0. => {
                if let Some(n) = self
                    .opponents
                    .iter()
                    .enumerate()
                    .filter(|(_, o)| dist(p.pos, o.pos) <= 4.5)
                    .min_by(|(_, a), (_, b)| dist(p.pos, a.pos).total_cmp(&dist(p.pos, b.pos)))
                    .map(|(n, _)| n)
                {
                    let opponent = &self.opponents[n];
                    if !p.profile.unlocked(n) {
                        Some("Defeat the previous stage first.".into())
                    } else if dist(p.pos, opponent.pos) > 4.5 {
                        Some(format!(
                            "Walk closer to {} to challenge them.",
                            opponent.name
                        ))
                    } else {
                        p.sitting = false;
                        p.duel = Some(n);
                        p.duel_progress = 0.5;
                        p.next_attack = self.time;
                        p.input = Input::default();
                        Some(format!("Fighting {}…", opponent.name))
                    }
                } else {
                    None
                }
            }
            "attack" if p.duel.is_some() && p.duel_progress > 0. && p.duel_progress < 1. => {
                if self.time < p.next_attack {
                    return;
                }
                p.next_attack = self.time + 0.049;
                p.duel_progress = (p.duel_progress
                    + p.profile.strength().max(1) as f64
                        / (self.opponents[p.duel.unwrap()].power as f64 * 12.))
                    .min(1.);
                p.attack_seq = p.attack_seq.wrapping_add(1);
                p.attack_target = p.duel;
                return;
            }
            "retreat" if p.duel.is_some() => {
                p.duel = None;
                p.duel_progress = 0.5;
                p.input = Input::default();
                Some("Fight abandoned. No rewards earned.".into())
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
        let mut trained = vec![];
        for (&id, p) in &mut self.players {
            if self.time - p.last_input > 0.3 {
                p.input = Input::default();
            }
            if let Some(n) = p.duel {
                if p.duel_progress < 1. {
                    p.duel_progress = (p.duel_progress - 0.12 * 0.05).max(0.);
                }
                if p.duel_progress >= 1. || p.duel_progress <= 0. {
                    let o = &self.opponents[n];
                    let won = p.duel_progress >= 1.;
                    let drop = won && p.profile.victory(n, random_roll() % 100);
                    finished.push((
                        id,
                        won,
                        if won {
                            format!(
                                "Defeated {}{}",
                                o.name,
                                if drop {
                                    " · Lootbox found! Open your inventory."
                                } else {
                                    ""
                                }
                            )
                        } else {
                            format!(
                                "{} pushed you back. Click faster or train for more power.",
                                o.name
                            )
                        },
                    ));
                    p.duel = None;
                }
                continue;
            }
            if p.sitting {
                if p.input.forward != 0. || p.input.right != 0. || p.input.jump {
                    p.sitting = false;
                } else {
                    if self.time >= p.next_train {
                        p.profile.train();
                        p.next_train = self.time + 1.;
                        trained.push(id);
                    }
                    continue;
                }
            }
            p.yaw = p.input.yaw;
            let f = p.input.forward;
            let r = p.input.right;
            let scale =
                if p.input.sprint { 7.4 } else { 4.8 } * 0.05 / (f * f + r * r).sqrt().max(1.);
            if p.input.jump && !p.jump_held && p.vertical_velocity == 0. {
                p.jump_origin = p.pos;
                p.vertical_velocity = 7.;
            }
            p.jump_held = p.input.jump;
            let dx = (-p.yaw.sin() * f + p.yaw.cos() * r) * scale;
            let dz = (-p.yaw.cos() * f - p.yaw.sin() * r) * scale;
            p.pos = if p.vertical_velocity != 0. {
                self.world
                    .jump_move(p.pos, dx, dz, &mut p.vertical_velocity)
            } else {
                self.world.move_player(p.pos, dx, dz)
            };
            if self.world.below_map(p.pos) {
                eprintln!("Out-of-map recovery: player={id}, jump_origin={:?}, position={:?}, velocity={}", p.jump_origin, p.pos, p.vertical_velocity);
                p.pos = [
                    -205.,
                    self.world.floor(-205., 150., -15.).unwrap_or(-20.),
                    150.,
                ];
                p.vertical_velocity = 0.;
                p.jump_held = p.input.jump;
                p.input = Input::default();
            }
        }
        for id in trained {
            self.remember(id);
        }
        for (id, won, text) in finished {
            self.remember(id);
            self.event(id, if won { "win" } else { "loss" }, text);
        }
    }
    pub fn snapshot(&self) -> serde_json::Value {
        serde_json::json!({"kind":"state","time":self.time,"players":self.players.values().collect::<Vec<_>>(),"opponents":self.opponents,"events":self.events,"items":ITEMS,"armors":ARMORS,"campfire":self.campfire()})
    }
}

fn random_roll() -> u32 {
    use std::hash::{BuildHasher, Hasher};
    std::collections::hash_map::RandomState::new()
        .build_hasher()
        .finish() as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    fn command(g: &mut Game, kind: &str) {
        g.command(
            1,
            serde_json::json!({"kind":kind,"tier":g.players[&1].profile.ascension+1}),
        );
    }
    fn ticks(g: &mut Game, n: usize) {
        for _ in 0..n {
            g.tick();
        }
    }
    fn challenge(g: &mut Game, n: usize, power: u64) {
        let p = g.players.get_mut(&1).unwrap();
        p.pos = g.opponents[n].pos;
        p.profile.power = power;
        p.input = Input::default();
        command(g, "challenge");
        assert_eq!(g.players[&1].duel, Some(n));
    }
    fn click_fight(g: &mut Game) -> usize {
        let mut count = 0;
        while g.players[&1].duel.is_some() && count < 120 {
            command(g, "attack");
            ticks(g, 5);
            count += 1;
        }
        assert_eq!(g.players[&1].duel, None);
        count
    }
    #[test]
    #[ignore = "requires imported Cordon assets"]
    fn cordon_stage_progression() {
        let root =
            std::env::var("CORDON_ASSETS").unwrap_or("D:/GameAssets/cordon-royale/web".into());
        let world = World::load(&std::path::Path::new(&root).join("collision.cform")).unwrap();
        let mut g = Game::new(world, BTreeMap::new());
        g.join(1, "test".into());
        command(&mut g, "train");
        assert_eq!(g.players[&1].profile.power, 0);
        g.players.get_mut(&1).unwrap().pos = [-206.817, -20., 142.435];
        command(&mut g, "sit");
        ticks(&mut g, 22);
        assert_eq!(g.profiles["test"].power, 1);
        g.command(1, serde_json::json!({"kind":"input","forward":1}));
        g.tick();
        assert!(!g.players[&1].sitting);
        g.players.get_mut(&1).unwrap().pos = g.opponents[0].reward_pos;
        g.command(1, serde_json::json!({"kind":"claim_exp","stage":0}));
        assert_eq!(
            g.players[&1].profile.exp, 0,
            "no free EXP without a victory"
        );
        challenge(&mut g, 0, 20);
        ticks(&mut g, 100);
        assert_eq!(g.events.last().unwrap().kind, "loss");
        assert!(g.players[&1].profile.run_wins.is_empty());
        challenge(&mut g, 0, 20);
        command(&mut g, "attack");
        let progress = g.players[&1].duel_progress;
        command(&mut g, "attack");
        assert_eq!(g.players[&1].duel_progress, progress, "spam ignored");
        command(&mut g, "ascend");
        assert_eq!(g.players[&1].profile.ascension, 0);
        ticks(&mut g, 5);
        let clicks = click_fight(&mut g);
        assert!(clicks < 20);
        assert!(g.players[&1].profile.unlocked(1));
        g.players.get_mut(&1).unwrap().pos = g.opponents[1].reward_pos;
        g.command(1, serde_json::json!({"kind":"claim_exp","stage":1}));
        assert_eq!(g.players[&1].profile.exp, 5);
        assert_eq!(g.players[&1].pos[0], -205.);
        assert!(!g.players[&1].profile.unlocked(1));
        assert!(g.players[&1].profile.trophies.contains(&0));
        g.players.get_mut(&1).unwrap().pos = g.opponents[1].reward_pos;
        g.command(1, serde_json::json!({"kind":"claim_exp","stage":1}));
        assert_eq!(g.players[&1].profile.exp, 5, "repeat claims need a new run");
        command(&mut g, "ascend");
        assert_eq!(g.players[&1].profile.exp, 0);
        assert_eq!(g.players[&1].profile.gain(), 3);
        g.command(1, serde_json::json!({"kind":"ascend","tier":1}));
        assert_eq!(
            g.players[&1].profile.ascension, 1,
            "duplicate upgrade cannot buy another tier"
        );
        command(&mut g, "ascend");
        assert_eq!(g.players[&1].profile.ascension, 1);
        let power = g.players[&1].profile.power;
        g.players.get_mut(&1).unwrap().pos = [-206.817, -20., 142.435];
        command(&mut g, "sit");
        ticks(&mut g, 22);
        assert_eq!(g.players[&1].profile.power, power + 3);
        challenge(&mut g, 0, 100);
        assert!(
            click_fight(&mut g) < clicks,
            "more power needs fewer clicks"
        );
        command(&mut g, "attack");
        let boxes = g.players[&1].profile.lootboxes;
        command(&mut g, "attack");
        assert_eq!(g.players[&1].profile.lootboxes, boxes);
        challenge(&mut g, 0, 100);
        command(&mut g, "retreat");
        assert_eq!(g.players[&1].duel, None);
        let saved = g.players[&1].profile.clone();
        g.players.get_mut(&1).unwrap().pos[1] = -57000.;
        g.players.get_mut(&1).unwrap().vertical_velocity = -100.;
        g.tick();
        assert!(g.players[&1].pos[1] > -30.);
        assert_eq!(g.players[&1].profile.power, saved.power);
        g.leave(1);
        g.join(1, "test".into());
        assert_eq!(g.players[&1].profile.ascension, 1);
        assert_eq!(g.players[&1].profile.run_wins, saved.run_wins);
    }
    #[test]
    #[ignore = "requires imported Cordon assets"]
    fn cordon_full_run_and_balance() {
        let root =
            std::env::var("CORDON_ASSETS").unwrap_or("D:/GameAssets/cordon-royale/web".into());
        let world = World::load(&std::path::Path::new(&root).join("collision.cform")).unwrap();
        let mut g = Game::new(world, BTreeMap::new());
        g.join(1, "balance".into());
        for stage in 0..5 {
            let power = g.opponents[stage].power;
            challenge(&mut g, stage, power);
            let clicks = click_fight(&mut g);
            assert_eq!(g.events.last().unwrap().kind, "win");
            eprintln!(
                "{} at equal power / 4 clicks per second: {} clicks, {:.2}s",
                g.opponents[stage].name,
                clicks,
                clicks as f64 * 0.25
            );
        }
        g.players.get_mut(&1).unwrap().pos = g.opponents[4].reward_pos;
        g.command(1, serde_json::json!({"kind":"claim_exp","stage":4}));
        assert_eq!(g.players[&1].profile.exp, 2500);
        assert!(g.players[&1].profile.run_wins.is_empty());
        assert_eq!(g.players[&1].profile.trophies.len(), 5);
        challenge(&mut g, 0, 10);
        let clicks = click_fight(&mut g);
        assert_eq!(g.events.last().unwrap().kind, "win");
        eprintln!(
            "Dog at half power / 4 clicks per second: {clicks} clicks, {:.2}s",
            clicks as f64 * 0.25
        );
        challenge(&mut g, 0, 200);
        assert_eq!(click_fight(&mut g), 1);
    }
}
