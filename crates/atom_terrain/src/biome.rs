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
