//! CPU terrain noise — multi-island Voronoi + FBM + coast noise + temperature.
//! Algorithm matches GPU `sdf_fill.wgsl` exactly.

use bevy::math::Vec2;

const ISLAND_SPACING: f32 = 500.0;
const ISLAND_PROB: f32 = 0.3;
const ISLAND_MIN_R: f32 = 150.0;
const ISLAND_MAX_R: f32 = 500.0;

const CELL_SIZE: f32 = 30.0;

fn hash_u32(x: u32) -> u32 {
    let mut h = x;
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    h
}
fn hash_2d(x: i32, z: i32, seed: u32) -> u32 {
    hash_u32(seed ^ ((x as u32).wrapping_mul(374761393)) ^ ((z as u32).wrapping_mul(668265263)))
}
fn hash_f32(h: u32) -> f32 {
    (h & 0x7fffff) as f32 / 0x7fffff_u32 as f32
}

fn noise_2d(x: f32, z: f32, seed: u32) -> f32 {
    let ix = x.floor() as i32;
    let iz = z.floor() as i32;
    let fx = x - ix as f32;
    let fz = z - iz as f32;
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sz = fz * fz * (3.0 - 2.0 * fz);
    let v00 = hash_f32(hash_2d(ix, iz, seed)) * 2.0 - 1.0;
    let v10 = hash_f32(hash_2d(ix + 1, iz, seed)) * 2.0 - 1.0;
    let v01 = hash_f32(hash_2d(ix, iz + 1, seed)) * 2.0 - 1.0;
    let v11 = hash_f32(hash_2d(ix + 1, iz + 1, seed)) * 2.0 - 1.0;
    let a = v00 + (v10 - v00) * sx;
    let b = v01 + (v11 - v01) * sx;
    a + (b - a) * sz
}

fn fbm(x: f32, z: f32, octaves: u32, persistence: f32, seed: u32) -> f32 {
    let mut value = 0.0_f32;
    let mut amplitude = 1.0_f32;
    let mut frequency = 1.0_f32;
    let mut max_val = 0.0_f32;
    for i in 0..octaves {
        value += noise_2d(x * frequency, z * frequency, seed.wrapping_add(i * 97)) * amplitude;
        max_val += amplitude;
        amplitude *= persistence;
        frequency *= 2.0;
    }
    value / max_val
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

struct IslandInfo {
    center: Vec2,
    radius: f32,
    exists: bool,
}

fn island_at(gx: i32, gz: i32, seed: u32) -> IslandInfo {
    let h = hash_2d(gx, gz, seed ^ 0x15_feed0);
    let exists = hash_f32(h) < ISLAND_PROB;
    let ox = (hash_f32(h) - 0.5) * ISLAND_SPACING * 0.6;
    let oz = (hash_f32(hash_u32(h)) - 0.5) * ISLAND_SPACING * 0.6;
    let r = ISLAND_MIN_R + hash_f32(hash_u32(h ^ 0x55)) * (ISLAND_MAX_R - ISLAND_MIN_R);
    IslandInfo {
        center: Vec2::new(
            gx as f32 * ISLAND_SPACING + ox,
            gz as f32 * ISLAND_SPACING + oz,
        ),
        radius: r,
        exists,
    }
}

fn island_falloff(pos: Vec2, seed: u32) -> f32 {
    let gx = (pos.x / ISLAND_SPACING).floor() as i32;
    let gz = (pos.y / ISLAND_SPACING).floor() as i32;
    let mut max_f = 0.0_f32;
    for dx in -1..=1 {
        for dz in -1..=1 {
            let island = island_at(gx + dx, gz + dz, seed);
            if !island.exists {
                continue;
            }
            let d_raw = pos.distance(island.center);
            let coast1 = fbm(pos.x * 0.018, pos.y * 0.018, 4, 0.55, seed ^ 0x7f7f_7f7f)
                * island.radius
                * 0.35;
            let coast2 =
                fbm(pos.x * 0.06, pos.y * 0.06, 2, 0.5, seed ^ 0x3f3f_3f3f) * island.radius * 0.15;
            let coast3 = fbm(pos.x * 0.003, pos.y * 0.003, 2, 0.5, seed ^ 0x5a5a_5a5a)
                * island.radius
                * 0.15;
            let d = (d_raw + coast1 + coast2 + coast3).max(0.0);
            if d < island.radius * 1.1 {
                let f = 1.0 - smoothstep(island.radius * 0.5, island.radius, d);
                max_f = max_f.max(f);
            }
        }
    }
    max_f
}

fn cell_seed_pos(cx: i32, cz: i32, seed: u32) -> Vec2 {
    let h = hash_2d(cx, cz, seed);
    let ox = (hash_f32(h) - 0.5) * CELL_SIZE * 0.7;
    let oz = (hash_f32(hash_u32(h)) - 0.5) * CELL_SIZE * 0.7;
    Vec2::new(cx as f32 * CELL_SIZE + ox, cz as f32 * CELL_SIZE + oz)
}

// ── Map edge ocean border ──
fn edge_falloff(pos: Vec2) -> f32 {
    let half = 2048.0_f32;
    let margin = 200.0_f32;
    let d = pos.x.abs().max(pos.y.abs()) / (half - margin);
    1.0 - smoothstep(0.85, 1.0, d)
}

/// 细胞温度（0..1，南热北冷纬度梯度 + FBM 扰动）。
pub fn cell_temperature(cx: i32, cz: i32, seed: u32) -> f32 {
    let pos = cell_seed_pos(cx, cz, seed);
    let lat = (pos.y + 2048.0) / 4096.0; // 0 at north, 1 at south
    let noise = fbm(pos.x * 0.0004, pos.y * 0.0004, 3, 0.55, seed ^ 0xdeed_deed);
    (lat * 0.55 + noise * 0.22 + 0.5).clamp(0.0, 1.0)
}

/// 细胞湿度（0..1）
pub fn cell_moisture(cx: i32, cz: i32, seed: u32) -> f32 {
    let pos = cell_seed_pos(cx, cz, seed);
    let wx = pos.x * 0.005 + 100.0;
    let wz = pos.y * 0.005 + 100.0;
    fbm(wx, wz, 3, 0.55, seed ^ 0xaaaa_aaaa) * 0.5 + 0.5
}

/// 世界坐标 (x,z) 处的地表高度（生态驱动台地）。
pub fn height_at(x: f32, z: f32) -> f32 {
    let wx = x * 0.004;
    let wy = z * 0.004;
    let raw_elev = (1.0 + fbm(wx, wy, 4, 0.55, 42))
        * 0.55
        * island_falloff(Vec2::new(x, z), 42)
        * edge_falloff(Vec2::new(x, z));
    let cx = (x / CELL_SIZE).floor() as i32;
    let cz = (z / CELL_SIZE).floor() as i32;
    let temp = cell_temperature(cx, cz, 42);
    let moist = cell_moisture(cx, cz, 42);
    let biome = crate::biome::surface_type_from_tme(temp, moist, raw_elev.max(0.0));
    let base = crate::biome::base_height(biome);
    let detail = fbm(x * 0.15, z * 0.15, 2, 0.5, 42u32.wrapping_add(777)) * 0.5;
    let micro = (hash_f32(hash_2d(
        (x * 5.0).floor() as i32,
        (z * 5.0).floor() as i32,
        999,
    )) - 0.5)
        * 0.3;
    base + detail + micro
}

/// 密度场：正值 = air，负值 = solid。
pub fn density_at(x: f32, y: f32, z: f32) -> f32 {
    y - height_at(x, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_at_smoke() {
        assert!(height_at(0.0, 0.0) >= -5.0 && height_at(0.0, 0.0) <= 30.0);
    }

    #[test]
    fn height_at_varied() {
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for i in 0..200 {
            let h = height_at(i as f32 * 2.0 - 200.0, i as f32 * 0.5 - 50.0);
            min = min.min(h);
            max = max.max(h);
        }
        assert!(max - min > 3.0, "range {min:.1}-{max:.1}");
    }

    #[test]
    fn island_exists() {
        let found = (-4..=4i32).any(|gx| (-4..=4i32).any(|gz| island_at(gx, gz, 42).exists));
        assert!(found);
    }

    #[test]
    fn temperature_latitude() {
        let t_north = cell_temperature(0, -50, 42); // far north
        let t_south = cell_temperature(0, 50, 42); // far south
        // South should be warmer than north
        assert!(
            t_south > t_north,
            "south {t_south:.2} should be warmer than north {t_north:.2}"
        );
    }

    #[test]
    fn density_at_is_y_minus_height() {
        for &(x, y, z) in &[(0.0, 0.0, 0.0), (3.5, 12.25, -7.0), (-100.0, -5.5, 200.0)] {
            let expected = y - height_at(x, z);
            assert!(
                (density_at(x, y, z) - expected).abs() < 1e-4,
                "density_at({x},{y},{z}) = {}，期望 {expected}",
                density_at(x, y, z)
            );
        }
    }
}
