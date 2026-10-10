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
    pub fn move_player(&self, pos: V, dx: f32, dz: f32) -> V {
        let mut p = pos;
        for delta in [[dx, 0., 0.], [0., 0., dz]] {
            let len = dist(delta, [0.; 3]);
            if len < 0.0001 {
                continue;
            }
            let d = mul(delta, 1. / len);
            let blocked = [0.55, 1.4]
                .iter()
                .any(|h| self.ray(add(p, [0., *h, 0.]), d, len + 0.38).is_some());
            if blocked {
                continue;
            }
            let q = add(p, delta);
            if let Some(y) = self.floor(q[0], q[2], p[1] + 0.55) {
                if (y - p[1]).abs() < 1.8 {
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
        };
        let direction = [0.8, 0., 0.6];
        assert!(world.ray([7.9, 0., 7.7], direction, 4.).is_some());
        assert!(world.ray([7.9, 0., 7.7], direction, 0.1).is_none());
    }
}
