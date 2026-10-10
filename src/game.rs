use crate::world::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const SPAWNS: [[f32; 2]; 8] = [
    [-205., 150.],
    [-180., 170.],
    [-230., 115.],
    [-150., 130.],
    [-250., 175.],
    [-190., 210.],
    [-130., 190.],
    [-265., 135.],
];
#[derive(Clone, Default, Deserialize)]
#[serde(default)]
pub struct Input {
    pub forward: f32,
    pub right: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub sprint: bool,
    pub fire: bool,
    pub reload: bool,
    pub interact: bool,
    pub heal: bool,
}
impl Input {
    pub fn valid(&self) -> bool {
        [self.forward, self.right, self.yaw, self.pitch]
            .iter()
            .all(|x| x.is_finite())
            && self.forward.abs() <= 1.
            && self.right.abs() <= 1.
            && self.pitch.abs() <= 1.55
    }
}
#[derive(Clone, Serialize)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub pos: V,
    pub yaw: f32,
    pub hp: f32,
    pub armor: f32,
    pub ammo: u32,
    pub reserve: u32,
    pub meds: u32,
    pub weapon: u8,
    pub kills: u32,
    pub bot: bool,
    pub reload: f32,
    pub search: f32,
    #[serde(skip)]
    pub input: Input,
    #[serde(skip)]
    pub last_input: f32,
    #[serde(skip)]
    pub next_shot: f32,
    #[serde(skip)]
    pub target: Option<usize>,
}
#[derive(Clone, Serialize)]
pub struct Stash {
    pub pos: V,
    pub open: bool,
}
#[derive(Clone, Serialize)]
pub struct Event {
    pub seq: u64,
    pub kind: String,
    pub text: String,
    pub who: u64,
    pub from: V,
    pub to: V,
}
#[derive(Serialize)]
pub struct Snapshot<'a> {
    pub kind: &'static str,
    pub time: f32,
    pub phase: &'a str,
    pub remaining: f32,
    pub radius: f32,
    pub center: [f32; 2],
    pub players: Vec<&'a Player>,
    pub stashes: &'a Vec<Stash>,
    pub events: &'a Vec<Event>,
    pub round: u32,
}
pub struct Game {
    pub world: World,
    pub players: BTreeMap<u64, Player>,
    pub stashes: Vec<Stash>,
    pub time: f32,
    pub phase: String,
    pub phase_at: f32,
    pub round: u32,
    pub events: Vec<Event>,
    seq: u64,
    random: u64,
}
impl Game {
    pub fn new(world: World) -> Self {
        let mut g = Self {
            world,
            players: BTreeMap::new(),
            stashes: Vec::new(),
            time: 0.,
            phase: "waiting".into(),
            phase_at: 0.,
            round: 1,
            events: Vec::new(),
            seq: 0,
            random: 0x912abc,
        };
        g.stashes = g.make_stashes();
        g
    }
    fn spawn(&self, id: u64) -> V {
        let [x, z] = SPAWNS[id as usize % SPAWNS.len()];
        [x, self.world.floor(x, z, -15.).unwrap_or(-20.), z]
    }
    fn make_stashes(&self) -> Vec<Stash> {
        let mut r = Vec::new();
        for [x, z] in SPAWNS {
            for (dx, dz) in [(3., 3.), (-8., 8.), (14., -10.)] {
                if let Some(y) = self.world.floor(x + dx, z + dz, -15.) {
                    r.push(Stash {
                        pos: [x + dx, y + 0.35, z + dz],
                        open: false,
                    })
                }
            }
        }
        r
    }
    pub fn join(&mut self, id: u64, bot: bool) {
        let hp = if self.phase == "active" { 0. } else { 100. };
        let pos = self.spawn(id);
        self.players.insert(
            id,
            Player {
                id,
                name: if bot {
                    format!("Scavenger {}", id - 1000)
                } else {
                    format!("Stalker {id}")
                },
                pos,
                yaw: 0.,
                hp,
                armor: 0.,
                ammo: 12,
                reserve: 60,
                meds: 1,
                weapon: 0,
                kills: 0,
                bot,
                reload: 0.,
                search: 0.,
                input: Input::default(),
                last_input: self.time,
                next_shot: 0.,
                target: None,
            },
        );
    }
    pub fn radius(&self) -> f32 {
        if self.phase != "active" {
            180.
        } else {
            (180. - (self.time - self.phase_at) * 0.78).max(12.)
        }
    }
    pub fn snapshot(&self) -> Snapshot<'_> {
        Snapshot {
            kind: "state",
            time: self.time,
            phase: &self.phase,
            remaining: match self.phase.as_str() {
                "waiting" => (15. - (self.time - self.phase_at)).max(0.),
                "active" => (220. - (self.time - self.phase_at)).max(0.),
                _ => (12. - (self.time - self.phase_at)).max(0.),
            },
            radius: self.radius(),
            center: [-195., 160.],
            players: self.players.values().collect(),
            stashes: &self.stashes,
            events: &self.events,
            round: self.round,
        }
    }
    pub fn event(&mut self, kind: &str, text: String, who: u64, from: V, to: V) {
        self.seq += 1;
        self.events.push(Event {
            seq: self.seq,
            kind: kind.into(),
            text,
            who,
            from,
            to,
        });
        if self.events.len() > 32 {
            self.events.remove(0);
        }
    }
    fn random(&mut self) -> u32 {
        self.random ^= self.random << 13;
        self.random ^= self.random >> 7;
        self.random ^= self.random << 17;
        self.random as u32
    }
    pub fn tick(&mut self) {
        let dt = 0.05;
        self.time += dt;
        if !self.players.values().any(|p| !p.bot) {
            if !self.players.is_empty() || self.phase != "waiting" {
                self.players.clear();
                self.stashes = self.make_stashes();
                self.events.clear();
                self.phase = "waiting".into();
            }
            self.phase_at = self.time;
            return;
        }
        for id in 1001..=1005 {
            if !self.players.contains_key(&id) {
                self.join(id, true)
            }
        }
        if self.phase == "waiting" && self.time - self.phase_at >= 15. {
            self.phase = "active".into();
            self.phase_at = self.time;
            self.event(
                "round",
                "The emission is closing in. Last stalker standing wins.".into(),
                0,
                [0.; 3],
                [0.; 3],
            );
        }
        if self.phase == "ended" {
            if self.time - self.phase_at > 12. {
                self.round += 1;
                self.phase = "waiting".into();
                self.phase_at = self.time;
                self.stashes = self.make_stashes();
                let ids: Vec<_> = self
                    .players
                    .iter()
                    .map(|(&id, p)| (id, p.name.clone(), p.bot))
                    .collect();
                for (id, name, bot) in ids {
                    self.join(id, bot);
                    self.players.get_mut(&id).unwrap().name = name;
                }
            }
            return;
        }
        let ids: Vec<_> = self.players.keys().copied().collect();
        let radius = self.radius();
        let active = self.phase == "active";
        for id in ids {
            let mut p = self.players.remove(&id).unwrap();
            if p.hp <= 0. {
                self.players.insert(id, p);
                continue;
            }
            if p.bot {
                let target = self
                    .players
                    .values()
                    .filter(|q| q.hp > 0.)
                    .min_by(|a, b| dist(p.pos, a.pos).total_cmp(&dist(p.pos, b.pos)));
                if let Some(q) = target {
                    let d = sub(q.pos, p.pos);
                    let range = dist(p.pos, q.pos);
                    p.input.yaw = (-d[0]).atan2(-d[2]);
                    p.input.pitch = 0.;
                    p.input.forward = if range > 16. { 0.6 } else { 0. };
                    p.input.right = (self.time * 0.3 + id as f32).sin() * 0.5;
                    p.input.fire = active && range < 55.;
                    p.input.reload = p.ammo == 0;
                }
                p.input.interact = true;
                p.last_input = self.time;
            }
            if self.time - p.last_input > 0.3 {
                p.input = Input::default()
            }
            p.yaw = p.input.yaw;
            let f = p.input.forward;
            let r = p.input.right;
            let norm = (f * f + r * r).sqrt().max(1.);
            let speed = if p.input.sprint { 7.4 } else { 4.8 };
            let dx = (-p.yaw.sin() * f + p.yaw.cos() * r) / norm * speed * dt;
            let dz = (-p.yaw.cos() * f - p.yaw.sin() * r) / norm * speed * dt;
            p.pos = self.world.move_player(p.pos, dx, dz);
            if p.reload > 0. {
                p.reload = (p.reload - dt).max(0.);
                if p.reload == 0. {
                    let n = (if p.weapon == 0 { 12 } else { 30 }) - p.ammo;
                    let n = n.min(p.reserve);
                    p.ammo += n;
                    p.reserve -= n;
                }
            }
            if p.input.reload
                && p.reload == 0.
                && p.reserve > 0
                && p.ammo < (if p.weapon == 0 { 12 } else { 30 })
            {
                p.reload = 1.7;
            }
            if p.input.heal && p.meds > 0 && p.hp < 100. {
                p.hp = (p.hp + 40.).min(100.);
                p.meds -= 1;
            }
            if p.input.interact {
                if let Some((n, _)) = self
                    .stashes
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| !s.open && dist(p.pos, s.pos) < 3.)
                    .min_by(|(_, a), (_, b)| dist(p.pos, a.pos).total_cmp(&dist(p.pos, b.pos)))
                {
                    let s = &self.stashes[n];
                    let origin = add(p.pos, [0., 1.5, 0.]);
                    let d = sub(s.pos, origin);
                    let len = dist(origin, s.pos);
                    if self
                        .world
                        .ray(origin, mul(d, 1. / len), len - 0.25)
                        .is_none()
                    {
                        if p.target == Some(n) {
                            p.search += dt
                        } else {
                            p.target = Some(n);
                            p.search = 0.
                        }
                        if p.search >= 1.2 {
                            self.stashes[n].open = true;
                            let roll = self.random() % 4;
                            p.reserve += 30;
                            p.meds += 1;
                            p.armor = (p.armor + 25.).min(100.);
                            if roll > 0 {
                                p.weapon = 1;
                                p.ammo = 30;
                            }
                            self.event(
                                "loot",
                                if roll > 0 {
                                    "Rifle · 30 rounds · armor · medkit"
                                } else {
                                    "Ammo · armor · medkit"
                                }
                                .into(),
                                id,
                                safe_pos(&p),
                                [0.; 3],
                            );
                            p.search = 0.;
                            p.target = None;
                        }
                    }
                } else {
                    p.search = 0.;
                    p.target = None;
                }
            } else {
                p.search = 0.;
                p.target = None;
            }
            if active && p.input.fire && p.reload == 0. && p.ammo > 0 && self.time >= p.next_shot {
                p.ammo -= 1;
                p.next_shot = self.time
                    + if p.bot {
                        0.8
                    } else if p.weapon == 0 {
                        0.32
                    } else {
                        0.12
                    };
                let origin = add(p.pos, [0., 1.6, 0.]);
                let pitch = p.input.pitch;
                let direction = [
                    -p.yaw.sin() * pitch.cos(),
                    pitch.sin(),
                    -p.yaw.cos() * pitch.cos(),
                ];
                let wall = self.world.ray(origin, direction, 160.).unwrap_or(160.);
                let mut hit = None;
                let mut nearest = wall;
                for (&other, q) in &self.players {
                    if q.hp <= 0. {
                        continue;
                    }
                    let rel = sub(add(q.pos, [0., 1., 0.]), origin);
                    let along = dot(rel, direction);
                    let perpendicular = dot(rel, rel) - along * along;
                    if along > 0. && along < nearest && perpendicular < 0.7 * 0.7 {
                        nearest = along;
                        hit = Some(other)
                    }
                }
                if let Some(target) = hit {
                    let q = self.players.get_mut(&target).unwrap();
                    let damage = if p.weapon == 0 { 24. } else { 19. };
                    let absorbed = q.armor.min(damage * 0.55);
                    q.armor -= absorbed;
                    q.hp = (q.hp - (damage - absorbed)).max(0.);
                    if q.hp == 0. {
                        p.kills += 1;
                        let victim = q.name.clone();
                        self.event(
                            "kill",
                            format!("{} eliminated {}", p.name, victim),
                            id,
                            origin,
                            [0.; 3],
                        );
                    }
                }
                self.event(
                    if hit.is_some() { "hit" } else { "shot" },
                    String::new(),
                    id,
                    origin,
                    add(origin, mul(direction, nearest)),
                );
            }
            if active && ((p.pos[0] + 195.).powi(2) + (p.pos[2] - 160.).powi(2)).sqrt() > radius {
                p.hp = (p.hp - dt * 8.).max(0.);
            }
            self.players.insert(id, p);
        }
        if active {
            let alive: Vec<_> = self.players.values().filter(|p| p.hp > 0.).collect();
            if alive.len() <= 1 || self.time - self.phase_at > 220. {
                let winner = alive
                    .iter()
                    .max_by(|a, b| a.hp.total_cmp(&b.hp))
                    .map(|p| p.name.clone())
                    .unwrap_or("Nobody".into());
                self.phase = "ended".into();
                self.phase_at = self.time;
                self.event(
                    "round",
                    format!("{winner} survived Cordon. Next round in 12 seconds."),
                    0,
                    [0.; 3],
                    [0.; 3],
                );
            }
        }
    }
}
fn safe_pos(p: &Player) -> V {
    p.pos
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_rejects_nonfinite_and_speed_hacks() {
        assert!(!Input {
            forward: 8.,
            ..Default::default()
        }
        .valid());
        assert!(!Input {
            yaw: f32::NAN,
            ..Default::default()
        }
        .valid());
        assert!(Input::default().valid());
    }
}
