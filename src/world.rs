use std::{collections::HashMap, fs, path::Path};
pub type V = [f32; 3];
pub fn sub(a: V, b: V) -> V {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
pub fn add(a: V, b: V) -> V {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
pub fn mul(a: V, s: f32) -> V {
    [a[0] * s, a[1] * s, a[2] * s]
}
pub fn dot(a: V, b: V) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
pub fn cross(a: V, b: V) -> V {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
pub fn dist(a: V, b: V) -> f32 {
    dot(sub(a, b), sub(a, b)).sqrt()
}
pub struct World {
    triangles: Vec<[V; 3]>,
    grid: HashMap<(i32, i32), Vec<usize>>,
    minimum_y: f32,
}
impl World {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let b = fs::read(path)?;
        let u = |p| u32::from_le_bytes(b[p..p + 4].try_into().unwrap());
        let f = |p| f32::from_le_bytes(b[p..p + 4].try_into().unwrap());
        if b.len() < 36 || u(0) != 4 {
            return Err("Expected X-Ray collision v4".into());
        }
        let nv = u(4) as usize;
        let nt = u(8) as usize;
        if b.len() != 36 + nv * 12 + nt * 16 {
            return Err("Invalid collision length".into());
        }
        let vertices: Vec<V> = (0..nv)
            .map(|i| {
                let p = 36 + i * 12;
                [f(p), f(p + 4), -f(p + 8)]
            })
            .collect();
        let mut world = Self {
            triangles: Vec::new(),
            grid: HashMap::new(),
            minimum_y: f32::INFINITY,
        };
        for i in 0..nt {
            let p = 36 + nv * 12 + i * 16;
            let ids = [u(p) as usize, u(p + 4) as usize, u(p + 8) as usize];
            if ids.iter().any(|&n| n >= nv) {
                return Err("Invalid collision index".into());
            }
            let tri = [vertices[ids[0]], vertices[ids[1]], vertices[ids[2]]];
            let xmin = tri.iter().map(|v| v[0]).fold(f32::INFINITY, f32::min);
            let xmax = tri.iter().map(|v| v[0]).fold(f32::NEG_INFINITY, f32::max);
            let zmin = tri.iter().map(|v| v[2]).fold(f32::INFINITY, f32::min);
            let zmax = tri.iter().map(|v| v[2]).fold(f32::NEG_INFINITY, f32::max);
            // The first room plays in southern Cordon; rendering retains the entire map.
            if xmax < -400. || xmin > 80. || zmax < -100. || zmin > 430. {
                continue;
            }
            world.minimum_y = world
                .minimum_y
                .min(tri.iter().map(|v| v[1]).fold(f32::INFINITY, f32::min));
            let id = world.triangles.len();
            world.triangles.push(tri);
            for x in (xmin / 8.).floor() as i32..=(xmax / 8.).floor() as i32 {
                for z in (zmin / 8.).floor() as i32..=(zmax / 8.).floor() as i32 {
                    world.grid.entry((x, z)).or_default().push(id);
                }
            }
        }
        Ok(world)
    }
    pub fn ray(&self, o: V, d: V, max: f32) -> Option<f32> {
        let mut nearest = max;
        let mut hit = false;
        let mut key = ((o[0] / 8.).floor() as i32, (o[2] / 8.).floor() as i32);
        let axis = |origin: f32, direction: f32, cell: i32| {
            if direction == 0. {
                return (0, f32::INFINITY, f32::INFINITY);
            }
            let step = if direction > 0. { 1 } else { -1 };
            let boundary = (cell + if step > 0 { 1 } else { 0 }) as f32 * 8.;
            (step, (boundary - origin) / direction, 8. / direction.abs())
        };
        let (sx, mut tx, dx) = axis(o[0], d[0], key.0);
        let (sz, mut tz, dz) = axis(o[2], d[2], key.1);
        loop {
            if let Some(ids) = self.grid.get(&key) {
                for &id in ids {
                    if let Some(t) = ray_triangle(o, d, self.triangles[id]) {
                        if t < nearest {
                            nearest = t;
                            hit = true
                        }
                    }
                }
            }
            if tx.min(tz) > nearest {
                break;
            }
            if tx < tz {
                key.0 += sx;
                tx += dx;
            } else {
                key.1 += sz;
                tz += dz;
            }
        }
        if hit {
            Some(nearest)
        } else {
            None
        }
    }
    pub fn floor(&self, x: f32, z: f32, top: f32) -> Option<f32> {
        self.ray([x, top, z], [0., -1., 0.], 120.).map(|t| top - t)
    }
    pub fn below_map(&self, pos: V) -> bool {
        pos[1] < self.minimum_y - 5.
    }
    pub fn jump_move(&self, pos: V, dx: f32, dz: f32, velocity: &mut f32) -> V {
        let mut p = pos;
        let steps = (dx.abs().max(dz.abs()) / 0.1).ceil().max(1.) as usize;
        for delta in
            (0..steps).flat_map(|_| [[dx / steps as f32, 0., 0.], [0., 0., dz / steps as f32]])
        {
            let len = dist(delta, [0.; 3]);
            // Keep the lowest horizontal sweep inside the landing probe
            // margin (2 cm), so rising ground cannot swallow the feet.
            if len > 0.0001
                && ![0.005, 0.55, 1.1, 1.7].iter().any(|h| {
                    self.ray(add(p, [0., *h, 0.]), mul(delta, 1. / len), len)
                        .is_some()
                })
            {
                p = add(p, delta);
            }
        }
        let dy = *velocity * 0.05;
        *velocity -= 18. * 0.05;
        if dy > 0. {
            if let Some(hit) = self.ray(add(p, [0., 1.7, 0.]), [0., 1., 0.], dy + 0.02) {
                p[1] += (hit - 0.02).max(0.);
                *velocity = -0.01;
            } else {
                p[1] += dy;
            }
        } else if let Some(y) = self
            .floor(p[0], p[2], p[1] + 0.02)
            .filter(|y| *y >= p[1] + dy)
        {
            p[1] = y;
            *velocity = 0.;
        } else {
            p[1] += dy;
        }
        p
    }
    pub fn move_player(&self, pos: V, dx: f32, dz: f32) -> V {
        let mut p = pos;
        let steps = (dx.abs().max(dz.abs()) / 0.1).ceil().max(1.) as usize;
        for delta in
            (0..steps).flat_map(|_| [[dx / steps as f32, 0., 0.], [0., 0., dz / steps as f32]])
        {
            let len = dist(delta, [0.; 3]);
            if len < 0.0001 {
                continue;
            }
            let d = mul(delta, 1. / len);
            let q = add(p, delta);
            if let Some(y) = self.floor(q[0], q[2], p[1] + 0.55) {
                let rise = y - p[1];
                if !(-1.8..=0.5).contains(&rise) {
                    continue;
                }
                // Check the body after stepping onto the destination tread. A
                // knee ray extended by the player radius hits the *next* riser
                // on narrow stairs and prevents reaching the current tread.
                let feet = p[1].max(y);
                let blocked = [0.55, 1.7]
                    .iter()
                    .any(|h| self.ray([p[0], feet + h, p[2]], d, len).is_some());
                let ceiling = rise > 0.
                    && self
                        .ray(add(p, [0., 1.7, 0.]), [0., 1., 0.], rise + 0.02)
                        .is_some();
                if !blocked && !ceiling {
                    p = [q[0], y, q[2]];
                }
            }
        }
        p
    }
}
pub fn ray_triangle(o: V, d: V, t: [V; 3]) -> Option<f32> {
    let e1 = sub(t[1], t[0]);
    let e2 = sub(t[2], t[0]);
    let p = cross(d, e2);
    let det = dot(e1, p);
    if det.abs() < 0.00001 {
        return None;
    }
    let inv = 1. / det;
    let s = sub(o, t[0]);
    let u = dot(s, p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return None;
    }
    let q = cross(s, e1);
    let v = dot(d, q) * inv;
    if v < 0. || u + v > 1. {
        return None;
    }
    let distance = dot(e2, q) * inv;
    if distance > 0.001 {
        Some(distance)
    } else {
        None
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn staircase(ceiling: bool, wall: bool) -> World {
        let mut triangles = Vec::new();
        let mut quad = |a, b, c, d| {
            triangles.push([a, b, c]);
            triangles.push([a, c, d]);
        };
        for i in 0..12 {
            let z = i as f32 * 0.2;
            let y = i as f32 * 0.3;
            quad([0., y, z], [2., y, z], [2., y, z + 0.2], [0., y, z + 0.2]);
            if i > 0 {
                quad([0., y - 0.3, z], [2., y - 0.3, z], [2., y, z], [0., y, z]);
            }
        }
        if ceiling {
            quad([0., 1.9, 0.], [2., 1.9, 0.], [2., 1.9, 2.4], [0., 1.9, 2.4]);
        }
        if wall {
            quad([0., 0., 0.6], [2., 0., 0.6], [2., 5., 0.6], [0., 5., 0.6]);
        }
        let ids = (0..triangles.len()).collect();
        World {
            triangles,
            grid: HashMap::from([((0, 0), ids)]),
            minimum_y: 0.,
        }
    }
    #[test]
    fn descending_jump_cannot_enter_rising_ground() {
        let w = World {
            triangles: vec![
                [[0., 0., 0.], [2., 0.8, 0.], [2., 0.8, 2.]],
                [[0., 0., 0.], [2., 0.8, 2.], [0., 0., 2.]],
            ],
            grid: HashMap::from([((0, 0), vec![0, 1])]),
            minimum_y: 0.,
        };
        let mut velocity = -0.1;
        let p = w.jump_move([0.5, 0.21, 0.5], 0.1, 0., &mut velocity);
        assert!(p[1] >= p[0] * 0.4 - 0.001, "feet entered slope: {p:?}");
    }
    #[test]
    fn jump_lands_and_stops_at_ceiling() {
        for ceiling in [false, true] {
            let w = staircase(ceiling, false);
            let mut p = [1., 0., 0.05];
            let mut velocity = 7.;
            let mut peak = 0_f32;
            for _ in 0..50 {
                if velocity != 0. {
                    p = w.jump_move(p, 0., 0., &mut velocity);
                }
                peak = peak.max(p[1]);
            }
            assert!(p[1].abs() < 0.001 && velocity == 0., "did not land: {p:?}");
            if ceiling {
                assert!(peak <= 0.2);
            } else {
                assert!(peak > 1.);
            }
        }
    }
    #[test]
    fn climbs_and_descends_narrow_stairs_at_walk_and_sprint_speed() {
        let w = staircase(false, false);
        for stride in [0.24, 0.37] {
            let mut p = [1., 0., 0.05];
            for _ in 0..8 {
                p = w.move_player(p, 0., stride);
            }
            assert!(p[2] > 1.8 && p[1] > 2.5, "stuck ascending: {p:?}");
            for _ in 0..8 {
                p = w.move_player(p, 0., -stride);
            }
            assert!(p[2] < 0.2 && p[1].abs() < 0.01, "stuck descending: {p:?}");
        }
    }
    #[test]
    fn stairs_do_not_bypass_walls_or_head_clearance() {
        for w in [staircase(true, false), staircase(false, true)] {
            let mut p = [1., 0., 0.05];
            for _ in 0..20 {
                p = w.move_player(p, 0., 0.37);
            }
            assert!(p[2] < 0.6, "walked through an obstruction: {p:?}");
        }
    }
    #[test]
    #[ignore = "requires local imported Cordon assets"]
    fn cordon_bush() {
        let root =
            std::env::var("CORDON_ASSETS").unwrap_or("D:/GameAssets/cordon-royale/web".into());
        let w = World::load(&Path::new(&root).join("collision.cform")).unwrap();
        let mut p = [-205., w.floor(-205., 149., -15.).unwrap(), 149.];
        for _ in 0..15 {
            p = w.move_player(p, 0., 0.24);
        }
        assert!(p[2] > 152., "blocked by the bush near camp: {p:?}");
    }
    #[test]
    #[ignore = "requires local imported Cordon assets"]
    fn cordon_stairs() {
        let root =
            std::env::var("CORDON_ASSETS").unwrap_or("D:/GameAssets/cordon-royale/web".into());
        let w = World::load(&Path::new(&root).join("collision.cform")).unwrap();
        for stride in [0.24, 0.37] {
            let mut actor = [-202.201, -22.418163, 130.18452];
            for _ in 0..(3.6_f32 / stride).ceil() as usize {
                actor = w.move_player(actor, 0., stride);
            }
            assert!(
                actor[2] > 133.5 && actor[1] > -19.8,
                "stuck ascending Cordon basement stairs: {actor:?}"
            );
            for _ in 0..(3.6_f32 / stride).ceil() as usize {
                actor = w.move_player(actor, 0., -stride);
            }
            assert!(
                actor[2] < 130.3 && actor[1] < -22.,
                "stuck descending Cordon basement stairs: {actor:?}"
            );
        }
    }
    #[test]
    #[ignore = "requires imported Cordon assets"]
    fn cordon_jumps_near_reported_fall() {
        let root =
            std::env::var("CORDON_ASSETS").unwrap_or("D:/GameAssets/cordon-royale/web".into());
        let w = World::load(&Path::new(&root).join("collision.cform")).unwrap();
        for x in -248..=-241 {
            for z in 98..=105 {
                let Some(y) = w.floor(x as f32, z as f32, 0.) else {
                    continue;
                };
                for direction in 0..8 {
                    let angle = direction as f32 * std::f32::consts::FRAC_PI_4;
                    let mut p = [x as f32, y, z as f32];
                    let mut velocity = 7.;
                    for _ in 0..100 {
                        p = w.jump_move(p, angle.sin() * 0.37, angle.cos() * 0.37, &mut velocity);
                        assert!(
                            !w.below_map(p),
                            "fell below Cordon at start {x},{z}, direction {direction}: {p:?}"
                        );
                        if velocity == 0. {
                            break;
                        }
                    }
                    assert_eq!(velocity, 0., "jump never landed from {x},{z}: {p:?}");
                }
            }
        }
    }
    #[test]
    fn ray_hits_both_sides_and_rejects_misses() {
        let t = [[0., 0., 0.], [2., 0., 0.], [0., 0., 2.]];
        assert_eq!(ray_triangle([0.5, 2., 0.5], [0., -1., 0.], t), Some(2.));
        assert_eq!(ray_triangle([3., 2., 3.], [0., -1., 0.], t), None);
    }
    #[test]
    fn ray_checks_cells_crossed_between_sample_points() {
        let triangle = [[8.1, -1., 7.], [8.1, 1., 7.], [8.1, 0., 7.99]];
        let world = World {
            triangles: vec![triangle],
            grid: HashMap::from([((1, 0), vec![0])]),
            minimum_y: -1.,
        };
        let direction = [0.8, 0., 0.6];
        assert!(world.ray([7.9, 0., 7.7], direction, 4.).is_some());
        assert!(world.ray([7.9, 0., 7.7], direction, 0.1).is_none());
    }
}
