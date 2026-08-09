//! 地表类型系统 — 温度 × 湿度 × 海拔 Whittaker 生物群落图。

/// 地表类型（8 种 + Ocean）
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SurfaceType {
    /// 雪地
    Snow = 0,
    /// 冻原
    Tundra = 1,
    /// 针叶林
    Taiga = 2,
    /// 温带森林
    Forest = 3,
    /// 草原
    Grassland = 4,
    /// 沙漠
    Desert = 5,
    /// 岩石
    Rock = 6,
    /// 沼泽
    Swamp = 7,
    /// 海洋
    Ocean = 8,
}

/// SurfaceType → 台地基础高度 (m)。
const BIOME_HEIGHT: [f32; 9] = [
    -2.0, // Ocean
    2.0,  // Desert
    3.0,  // Swamp
    5.0,  // Grassland
    8.0,  // Forest
    12.0, // Taiga
    15.0, // Tundra
    20.0, // Snow
    18.0, // Rock
];

/// 生态 → 台地基础高度 (m)。
pub fn base_height(st: SurfaceType) -> f32 {
    BIOME_HEIGHT[st as usize]
}
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

fn cell_seed_pos(cx: i32, cz: i32, seed: u32) -> bevy::math::Vec2 {
    let h = hash_2d(cx, cz, seed);
    let ox = (hash_f32(h) - 0.5) * CELL_SIZE * 0.7;
    let oz = (hash_f32(hash_u32(h)) - 0.5) * CELL_SIZE * 0.7;
    bevy::math::Vec2::new(cx as f32 * CELL_SIZE + ox, cz as f32 * CELL_SIZE + oz)
}

struct IslandInfo {
    center: bevy::math::Vec2,
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
        center: bevy::math::Vec2::new(
            gx as f32 * ISLAND_SPACING + ox,
            gz as f32 * ISLAND_SPACING + oz,
        ),
        radius: r,
        exists,
    }
}

fn island_falloff(pos: bevy::math::Vec2, seed: u32) -> f32 {
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

// ── Map edge ocean border ──
fn edge_falloff(pos: bevy::math::Vec2) -> f32 {
    let half = 2048.0_f32;
    let margin = 200.0_f32;
    let d = pos.x.abs().max(pos.y.abs()) / (half - margin);
    1.0 - smoothstep(0.85, 1.0, d)
}

/// 细胞高度（0..1）
pub fn cell_elevation(cx: i32, cz: i32, seed: u32) -> f32 {
    let pos = cell_seed_pos(cx, cz, seed);
    let wx = pos.x * 0.004;
    let wz = pos.y * 0.004;
    let elev = (1.0 + fbm(wx, wz, 4, 0.55, seed)) * 0.55;
    let falloff = island_falloff(pos, seed) * edge_falloff(pos);
    let mut e = elev * falloff;
    if e > 0.55 {
        let ridge = (e - 0.55) / 0.35;
        e = (e + ridge * ridge * 0.25).min(1.0);
    }
    e.max(0.0)
}

/// 细胞温度（0..1，南热北冷）
pub fn cell_temperature(cx: i32, cz: i32, seed: u32) -> f32 {
    let pos = cell_seed_pos(cx, cz, seed);
    let lat = (pos.y + 2048.0) / 4096.0;
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

/// Whittaker 图：温度 × 湿度 × 海拔 → 地表类型
pub fn surface_type_from_tme(temp: f32, moist: f32, elev: f32) -> SurfaceType {
    if elev < 0.05 {
        return SurfaceType::Ocean;
    }
    if elev > 0.7 {
        return if temp < 0.3 {
            SurfaceType::Snow
        } else {
            SurfaceType::Rock
        };
    }
    if temp < 0.25 {
        if moist < 0.33 {
            SurfaceType::Tundra
        } else if moist < 0.66 {
            SurfaceType::Taiga
        } else {
            SurfaceType::Snow
        }
    } else if temp < 0.45 {
        if moist < 0.33 {
            SurfaceType::Grassland
        } else if moist < 0.66 {
            SurfaceType::Taiga
        } else {
            SurfaceType::Forest
        }
    } else if temp < 0.65 {
        if moist < 0.2 {
            SurfaceType::Desert
        } else if moist < 0.5 {
            SurfaceType::Grassland
        } else if moist < 0.8 {
            SurfaceType::Forest
        } else {
            SurfaceType::Swamp
        }
    } else {
        if moist < 0.15 {
            SurfaceType::Desert
        } else if moist < 0.4 {
            SurfaceType::Grassland
        } else {
            SurfaceType::Swamp
        }
    }
}

/// 世界坐标 → 地表类型
pub fn surface_at(x: f32, z: f32, seed: u32) -> SurfaceType {
    let cx = (x / CELL_SIZE).floor() as i32;
    let cz = (z / CELL_SIZE).floor() as i32;
    let e = cell_elevation(cx, cz, seed);
    let t = cell_temperature(cx, cz, seed);
    let m = cell_moisture(cx, cz, seed);
    surface_type_from_tme(t, m, e)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// f32 近似相等（避免 clippy float_cmp）
    fn assert_approx(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-5,
            "expected {a} ≈ {b} but diff = {}",
            (a - b).abs()
        );
    }

    // ── base_height ──

    /// 锁定当前 as-coded 映射（BIOME_HEIGHT 数组序与 SurfaceType 判别值序不一致，
    /// 此处按索引访问的当前行为断言；若后续修正数组序需同步更新本测试）。
    #[test]
    fn base_height_maps_all_surface_types() {
        assert_eq!(base_height(SurfaceType::Snow), -2.0);
        assert_eq!(base_height(SurfaceType::Tundra), 2.0);
        assert_eq!(base_height(SurfaceType::Taiga), 3.0);
        assert_eq!(base_height(SurfaceType::Forest), 5.0);
        assert_eq!(base_height(SurfaceType::Grassland), 8.0);
        assert_eq!(base_height(SurfaceType::Desert), 12.0);
        assert_eq!(base_height(SurfaceType::Rock), 15.0);
        assert_eq!(base_height(SurfaceType::Swamp), 20.0);
        assert_eq!(base_height(SurfaceType::Ocean), 18.0);
    }

    // ── hash 函数 ──

    #[test]
    fn hash_u32_deterministic_and_avalanches() {
        assert_eq!(hash_u32(42), hash_u32(42));
        assert_eq!(hash_u32(0), hash_u32(0));
        assert_eq!(hash_u32(u32::MAX), hash_u32(u32::MAX));
        let vals = [0u32, 1, 2, 3, 0xffff, u32::MAX, 123456789];
        let hashes: Vec<u32> = vals.iter().map(|&v| hash_u32(v)).collect();
        for i in 0..hashes.len() {
            for j in (i + 1)..hashes.len() {
                assert_ne!(hashes[i], hashes[j], "hash_u32 collision for {i} vs {j}");
            }
        }
    }

    #[test]
    fn hash_2d_deterministic_and_distinct() {
        assert_eq!(hash_2d(1, 2, 42), hash_2d(1, 2, 42));
        assert_eq!(hash_2d(-3, -7, 0), hash_2d(-3, -7, 0));
        assert_ne!(hash_2d(0, 0, 42), hash_2d(1, 0, 42));
        assert_ne!(hash_2d(0, 0, 42), hash_2d(0, 1, 42));
        assert_ne!(hash_2d(0, 0, 42), hash_2d(0, 0, 43));
        // 负坐标 + 极值 seed 不 panic、确定性
        assert_eq!(
            hash_2d(i32::MIN, i32::MAX, u32::MAX),
            hash_2d(i32::MIN, i32::MAX, u32::MAX)
        );
    }

    #[test]
    fn hash_f32_in_unit_range() {
        for h in [0u32, 1, 0x123456, 0x7fffff, 0xffffff, u32::MAX, 987654321] {
            let v = hash_f32(h);
            assert!((0.0..=1.0).contains(&v), "hash_f32({h}) = {v} out of [0,1]");
        }
        assert_eq!(hash_f32(0), 0.0);
        assert_eq!(hash_f32(0x7fffff), 1.0);
    }

    // ── noise_2d / fbm ──

    #[test]
    fn noise_2d_range_and_deterministic() {
        for &(x, z) in &[
            (0.0, 0.0),
            (12.5, -3.25),
            (1.0, 1.0),
            (0.0, 1000.0),
            (-500.0, -500.0),
        ] {
            for seed in [0u32, 42, u32::MAX] {
                let n = noise_2d(x, z, seed);
                assert!((-1.0..=1.0).contains(&n), "noise_2d({x},{z},{seed}) = {n}");
                assert_eq!(n, noise_2d(x, z, seed));
            }
        }
    }

    #[test]
    fn fbm_single_octave_matches_noise() {
        assert_approx(fbm(1.5, -2.5, 1, 0.5, 42), noise_2d(1.5, -2.5, 42));
    }

    #[test]
    fn fbm_range_and_deterministic() {
        for &(x, z) in &[(0.0, 0.0), (100.0, -50.0), (-3.5, 7.25)] {
            for seed in [0u32, 42, u32::MAX] {
                let v = fbm(x, z, 4, 0.55, seed);
                assert!((-1.0..=1.0).contains(&v), "fbm({x},{z},{seed}) = {v}");
                assert_eq!(v, fbm(x, z, 4, 0.55, seed));
            }
        }
    }

    // ── smoothstep ──

    #[test]
    fn smoothstep_edges_and_midpoint() {
        assert_eq!(smoothstep(0.0, 1.0, 0.0), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 1.0), 1.0);
        assert_eq!(smoothstep(0.0, 1.0, -10.0), 0.0);
        assert_eq!(smoothstep(0.0, 1.0, 10.0), 1.0);
        assert_approx(smoothstep(0.0, 1.0, 0.5), 0.5);
        // 单调递增
        let mut prev = 0.0;
        for i in 0..=10 {
            let s = smoothstep(0.0, 1.0, i as f32 / 10.0);
            assert!(s >= prev - 1e-6, "smoothstep not monotonic at {i}");
            prev = s;
        }
    }

    // ── cell_seed_pos ──

    #[test]
    fn cell_seed_pos_within_cell_bounds() {
        let half = CELL_SIZE * 0.7 * 0.5; // 最大偏移 10.5
        for &(cx, cz) in &[(0, 0), (3, -2), (-5, 7), (100, -100)] {
            for seed in [0u32, 42] {
                let pos = cell_seed_pos(cx, cz, seed);
                assert!(
                    (pos.x - cx as f32 * CELL_SIZE).abs() <= half + 1e-5,
                    "cell ({cx},{cz}) seed {seed} x offset too large: {}",
                    pos.x - cx as f32 * CELL_SIZE
                );
                assert!(
                    (pos.y - cz as f32 * CELL_SIZE).abs() <= half + 1e-5,
                    "cell ({cx},{cz}) seed {seed} z offset too large: {}",
                    pos.y - cz as f32 * CELL_SIZE
                );
            }
        }
    }

    #[test]
    fn cell_seed_pos_deterministic() {
        let a = cell_seed_pos(5, -3, 42);
        let b = cell_seed_pos(5, -3, 42);
        assert_approx(a.x, b.x);
        assert_approx(a.y, b.y);
    }

    // ── island_at ──

    #[test]
    fn island_radius_within_bounds() {
        let half = ISLAND_SPACING * 0.6 * 0.5;
        for &(gx, gz) in &[(0, 0), (2, -1), (-3, 4)] {
            for seed in [0u32, 42, u32::MAX] {
                let island = island_at(gx, gz, seed);
                assert!(
                    (ISLAND_MIN_R..=ISLAND_MAX_R).contains(&island.radius),
                    "radius {} out of [{ISLAND_MIN_R},{ISLAND_MAX_R}]",
                    island.radius
                );
                assert!(
                    (island.center.x - gx as f32 * ISLAND_SPACING).abs() <= half + 1e-5,
                    "center x offset too large"
                );
                assert!(
                    (island.center.y - gz as f32 * ISLAND_SPACING).abs() <= half + 1e-5,
                    "center y offset too large"
                );
            }
        }
    }

    #[test]
    fn island_deterministic() {
        let a = island_at(3, -4, 42);
        let b = island_at(3, -4, 42);
        assert_eq!(a.exists, b.exists);
        assert_approx(a.radius, b.radius);
        assert_approx(a.center.x, b.center.x);
        assert_approx(a.center.y, b.center.y);
    }

    #[test]
    fn island_exists_some_and_some_not() {
        let mut any_exists = false;
        let mut any_missing = false;
        for gx in -10..10 {
            for gz in -10..10 {
                if island_at(gx, gz, 42).exists {
                    any_exists = true;
                } else {
                    any_missing = true;
                }
            }
        }
        assert!(any_exists, "no island found in 20x20 grid");
        assert!(any_missing, "all cells are islands (probability 0.3)");
    }

    // ── island_falloff / edge_falloff ──

    #[test]
    fn island_falloff_unit_range() {
        for &(x, z) in &[
            (0.0, 0.0),
            (100.0, -200.0),
            (1500.0, 1500.0),
            (-800.0, 300.0),
        ] {
            for seed in [0u32, 42, u32::MAX] {
                let f = island_falloff(bevy::math::Vec2::new(x, z), seed);
                assert!(
                    (0.0..=1.0).contains(&f),
                    "island_falloff({x},{z},{seed}) = {f}"
                );
            }
        }
    }

    #[test]
    fn island_falloff_deterministic() {
        let a = island_falloff(bevy::math::Vec2::new(123.0, -456.0), 42);
        let b = island_falloff(bevy::math::Vec2::new(123.0, -456.0), 42);
        assert_eq!(a, b);
    }

    #[test]
    fn edge_falloff_center_one_far_zero() {
        assert_eq!(edge_falloff(bevy::math::Vec2::new(0.0, 0.0)), 1.0);
        // 超出地图边缘 → 0
        assert_eq!(edge_falloff(bevy::math::Vec2::new(100_000.0, 0.0)), 0.0);
        assert_eq!(
            edge_falloff(bevy::math::Vec2::new(-100_000.0, 100_000.0)),
            0.0
        );
        // 对称性
        let a = edge_falloff(bevy::math::Vec2::new(800.0, -400.0));
        let b = edge_falloff(bevy::math::Vec2::new(-800.0, 400.0));
        assert_eq!(a, b);
        // 范围内单调下降（d = |x|/1848，过渡区约 x ∈ [1570, 1848]）
        let near = edge_falloff(bevy::math::Vec2::new(1500.0, 0.0));
        let mid = edge_falloff(bevy::math::Vec2::new(1700.0, 0.0));
        let far = edge_falloff(bevy::math::Vec2::new(1800.0, 0.0));
        assert!(
            near > mid && mid > far,
            "edge falloff not decreasing: {near} {mid} {far}"
        );
    }

    // ── cell_elevation / cell_temperature / cell_moisture ──

    #[test]
    fn cell_elevation_unit_range() {
        for &(cx, cz) in &[(0, 0), (3, -2), (-5, 7), (100, -100), (-50, -50)] {
            for seed in [0u32, 42, u32::MAX] {
                let e = cell_elevation(cx, cz, seed);
                assert!(
                    (0.0..=1.0).contains(&e),
                    "cell_elevation({cx},{cz},{seed}) = {e}"
                );
            }
        }
    }

    #[test]
    fn cell_elevation_deterministic() {
        let a = cell_elevation(7, -9, 0);
        let b = cell_elevation(7, -9, 0);
        assert_eq!(a, b);
    }

    #[test]
    fn cell_elevation_zero_far_beyond_edge() {
        // 远离地图边缘 → edge falloff 归零 → 高度 0（海洋）
        for &(cx, cz) in &[(100_000, 0), (0, 100_000), (-100_000, -100_000)] {
            let e = cell_elevation(cx, cz, 42);
            assert!(
                e.abs() < 1e-6,
                "cell_elevation({cx},{cz}) = {e}, expected ~0"
            );
        }
    }

    #[test]
    fn cell_temperature_unit_range() {
        for &(cx, cz) in &[(0, 0), (10, -10), (-3, 5), (1000, 1000), (-1000, 0)] {
            for seed in [0u32, 42, u32::MAX] {
                let t = cell_temperature(cx, cz, seed);
                assert!(
                    (0.0..=1.0).contains(&t),
                    "cell_temperature({cx},{cz},{seed}) = {t}"
                );
            }
        }
    }

    #[test]
    fn cell_temperature_deterministic() {
        let a = cell_temperature(-4, 6, 42);
        let b = cell_temperature(-4, 6, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn cell_moisture_unit_range() {
        for &(cx, cz) in &[(0, 0), (5, 5), (-8, 3), (500, -200)] {
            for seed in [0u32, 42, u32::MAX] {
                let m = cell_moisture(cx, cz, seed);
                assert!(
                    (0.0..=1.0).contains(&m),
                    "cell_moisture({cx},{cz},{seed}) = {m}"
                );
            }
        }
    }

    #[test]
    fn cell_moisture_deterministic() {
        let a = cell_moisture(2, -3, 0);
        let b = cell_moisture(2, -3, 0);
        assert_eq!(a, b);
    }

    // ── surface_type_from_tme ──

    #[test]
    fn tme_ocean_when_elevation_below_threshold() {
        assert_eq!(surface_type_from_tme(0.0, 0.0, 0.0), SurfaceType::Ocean);
        assert_eq!(surface_type_from_tme(0.99, 0.99, 0.049), SurfaceType::Ocean);
        // 边界：elev = 0.05 不是海洋
        assert_eq!(surface_type_from_tme(0.0, 0.0, 0.05), SurfaceType::Tundra);
    }

    #[test]
    fn tme_high_elevation_snow_or_rock() {
        assert_eq!(surface_type_from_tme(0.0, 0.5, 0.71), SurfaceType::Snow);
        assert_eq!(surface_type_from_tme(0.299, 0.5, 1.0), SurfaceType::Snow);
        // temp >= 0.3 → Rock
        assert_eq!(surface_type_from_tme(0.3, 0.5, 0.71), SurfaceType::Rock);
        assert_eq!(surface_type_from_tme(0.99, 0.0, 0.71), SurfaceType::Rock);
    }

    #[test]
    fn tme_cold_band() {
        // temp < 0.25
        assert_eq!(surface_type_from_tme(0.0, 0.32, 0.3), SurfaceType::Tundra);
        assert_eq!(surface_type_from_tme(0.24, 0.65, 0.3), SurfaceType::Taiga);
        assert_eq!(surface_type_from_tme(0.1, 0.66, 0.3), SurfaceType::Snow);
        assert_eq!(surface_type_from_tme(0.1, 1.0, 0.3), SurfaceType::Snow);
    }

    #[test]
    fn tme_cool_band() {
        // 0.25 <= temp < 0.45
        assert_eq!(
            surface_type_from_tme(0.25, 0.32, 0.3),
            SurfaceType::Grassland
        );
        assert_eq!(surface_type_from_tme(0.44, 0.65, 0.3), SurfaceType::Taiga);
        assert_eq!(surface_type_from_tme(0.3, 0.66, 0.3), SurfaceType::Forest);
    }

    #[test]
    fn tme_warm_band() {
        // 0.45 <= temp < 0.65
        assert_eq!(surface_type_from_tme(0.45, 0.19, 0.3), SurfaceType::Desert);
        assert_eq!(
            surface_type_from_tme(0.64, 0.49, 0.3),
            SurfaceType::Grassland
        );
        assert_eq!(surface_type_from_tme(0.5, 0.79, 0.3), SurfaceType::Forest);
        assert_eq!(surface_type_from_tme(0.6, 0.8, 0.3), SurfaceType::Swamp);
    }

    #[test]
    fn tme_hot_band() {
        // temp >= 0.65
        assert_eq!(surface_type_from_tme(0.65, 0.14, 0.3), SurfaceType::Desert);
        assert_eq!(
            surface_type_from_tme(0.9, 0.39, 0.3),
            SurfaceType::Grassland
        );
        assert_eq!(surface_type_from_tme(0.7, 0.4, 0.3), SurfaceType::Swamp);
    }

    #[test]
    fn tme_band_boundaries_locked() {
        // 湿度边界逐一锁定（取等属于更高分区）
        assert_eq!(surface_type_from_tme(0.1, 0.33, 0.3), SurfaceType::Taiga);
        assert_eq!(surface_type_from_tme(0.1, 0.66, 0.3), SurfaceType::Snow);
        assert_eq!(surface_type_from_tme(0.3, 0.33, 0.3), SurfaceType::Taiga);
        assert_eq!(surface_type_from_tme(0.3, 0.66, 0.3), SurfaceType::Forest);
        assert_eq!(surface_type_from_tme(0.5, 0.2, 0.3), SurfaceType::Grassland);
        assert_eq!(surface_type_from_tme(0.5, 0.5, 0.3), SurfaceType::Forest);
        assert_eq!(surface_type_from_tme(0.5, 0.8, 0.3), SurfaceType::Swamp);
        assert_eq!(
            surface_type_from_tme(0.8, 0.15, 0.3),
            SurfaceType::Grassland
        );
        assert_eq!(surface_type_from_tme(0.8, 0.4, 0.3), SurfaceType::Swamp);
        // 温度边界：temp 恰好 0.25/0.45/0.65 属于下一带
        assert_eq!(
            surface_type_from_tme(0.25, 0.0, 0.3),
            SurfaceType::Grassland
        );
        assert_eq!(surface_type_from_tme(0.45, 0.0, 0.3), SurfaceType::Desert);
        assert_eq!(surface_type_from_tme(0.65, 0.0, 0.3), SurfaceType::Desert);
        // elev 边界：0.05 非海洋、0.7 非高山
        assert_eq!(surface_type_from_tme(0.0, 0.0, 0.05), SurfaceType::Tundra);
        assert_eq!(surface_type_from_tme(0.0, 0.0, 0.7), SurfaceType::Tundra);
    }

    // ── surface_at ──

    #[test]
    fn surface_at_deterministic() {
        for &(x, z) in &[(0.0, 0.0), (123.5, -87.25), (-45.0, 300.0)] {
            for seed in [0u32, 42] {
                assert_eq!(surface_at(x, z, seed), surface_at(x, z, seed));
            }
        }
    }

    #[test]
    fn surface_at_ocean_far_off_map() {
        // 远离地图边界 → elevation 0 → 海洋
        assert_eq!(surface_at(100_000.0, 0.0, 42), SurfaceType::Ocean);
        assert_eq!(surface_at(-100_000.0, 0.0, 42), SurfaceType::Ocean);
        assert_eq!(surface_at(0.0, 100_000.0, 42), SurfaceType::Ocean);
    }

    #[test]
    fn surface_at_valid_type_on_map() {
        let t = surface_at(0.0, 0.0, 42);
        assert!(
            matches!(
                t,
                SurfaceType::Snow
                    | SurfaceType::Tundra
                    | SurfaceType::Taiga
                    | SurfaceType::Forest
                    | SurfaceType::Grassland
                    | SurfaceType::Desert
                    | SurfaceType::Rock
                    | SurfaceType::Swamp
                    | SurfaceType::Ocean
            ),
            "unexpected surface type {t:?}"
        );
    }
}
