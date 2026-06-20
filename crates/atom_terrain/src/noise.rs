//! CPU 端 OpenSimplex 2D 噪声 + FBM，与 GPU `noise::*` composable module 逐位一致。
//! 密度场 = y - height_at(x,z)，正值 = air，负值 = solid。

use std::f32::consts::FRAC_1_SQRT_2;

use bevy::math::{IVec2, UVec4, Vec2};

const DIAG: f32 = FRAC_1_SQRT_2;
#[allow(clippy::excessive_precision)]
const STRETCH_CONSTANT_2D: f32 = -0.211324865405187;
#[allow(clippy::excessive_precision)]
const SQUISH_CONSTANT_2D: f32 = 0.366025403784439;
const NORM_CONSTANT_2D: f32 = 1.0 / 14.0;
const TABLE_SIZE: usize = 256;

const GRAD2: [Vec2; 8] = [
    Vec2::new(1.0, 0.0),
    Vec2::new(-1.0, 0.0),
    Vec2::new(0.0, 1.0),
    Vec2::new(0.0, -1.0),
    Vec2::new(DIAG, DIAG),
    Vec2::new(-DIAG, DIAG),
    Vec2::new(DIAG, -DIAG),
    Vec2::new(-DIAG, -DIAG),
];

fn xorshift_128_with_seed(state: &mut UVec4) -> u32 {
    let t = state.x ^ (state.x << 11);
    state.x = state.y;
    state.y = state.z;
    state.z = state.w;
    state.w = state.w ^ (state.w >> 19) ^ t ^ (t >> 8);
    state.w
}

/// XorShift128 随机数生成器生成的置换表，用于噪声哈希
/// 生成见 `open_simplex_seed.wgsl` 的 GPU 端等价实现
pub fn generate_permutation_table(seed: u32) -> [u32; TABLE_SIZE] {
    let mut state = UVec4::new(seed, seed ^ 0x9E37_79B9, seed ^ 0x6A09_E667, seed ^ 0xBB67_AE85);
    for _ in 0..8 {
        xorshift_128_with_seed(&mut state);
    }
    let mut table = [0u32; TABLE_SIZE];
    let mut idx = 0;
    while idx < TABLE_SIZE {
        let val = xorshift_128_with_seed(&mut state) as usize % TABLE_SIZE;
        if !table[..idx].contains(&(val as u32)) {
            table[idx] = val as u32;
            idx += 1;
        }
    }
    table
}

fn hash_21(perm: &[u32; TABLE_SIZE], to_hash: IVec2) -> u32 {
    let idx = (to_hash.x as u32 ^ to_hash.y as u32) as usize % TABLE_SIZE;
    perm[idx]
}

fn surflet_2d(index: usize, point: Vec2) -> f32 {
    let grad = GRAD2[index % 8];
    let t = 1.0 - point.length_squared();
    if t > 0.0 {
        t * t * t * t * grad.dot(point)
    } else {
        0.0
    }
}

fn _contribute_2d(
    perm: &[u32; TABLE_SIZE],
    stretched_floor: Vec2,
    rel_pos: Vec2,
    x: f32,
    y: f32,
) -> f32 {
    let displacement = 1.0 - rel_pos.x - rel_pos.y;
    let vertex = stretched_floor + Vec2::new(x, y);
    let hash = hash_21(perm, vertex.as_ivec2()) as usize;
    let attn = surflet_2d(
        hash,
        rel_pos
            - Vec2::new(
                x - displacement * SQUISH_CONSTANT_2D,
                y - displacement * SQUISH_CONSTANT_2D,
            ),
    );
    attn * NORM_CONSTANT_2D
}

/// OpenSimplex 2D 噪声基础函数，值域 [-1, 1]。
/// 与 `open_simplex.wgsl` 中 `open_simplex_2d` 逐位一致。
pub fn open_simplex_2d(point: Vec2, perm: &[u32; TABLE_SIZE]) -> f32 {
    let stretch_offset = (point.x + point.y) * -STRETCH_CONSTANT_2D;
    let stretched = point + Vec2::splat(stretch_offset);
    let floor = stretched.floor();
    let squish_offset = (floor.x + floor.y) * SQUISH_CONSTANT_2D;
    let origin = floor - Vec2::splat(squish_offset);
    let rel = point - origin;
    let xsv = (rel.x > rel.y) as i32 as f32;
    let ysv = 1.0 - xsv;
    let dx0 = rel.x + STRETCH_CONSTANT_2D;
    let dy0 = rel.y + STRETCH_CONSTANT_2D;
    let dx_ext = rel.x + STRETCH_CONSTANT_2D - 1.0;
    let dy_ext = rel.y + STRETCH_CONSTANT_2D - 1.0;
    let attn0 = surflet_2d(hash_21(perm, floor.as_ivec2()) as usize, Vec2::new(dx0, dy0));
    let attn1 = surflet_2d(
        hash_21(perm, (floor + Vec2::new(xsv, ysv)).as_ivec2()) as usize,
        Vec2::new(dx0 - xsv + SQUISH_CONSTANT_2D, dy0 - ysv + SQUISH_CONSTANT_2D),
    );
    let attn2 = surflet_2d(
        hash_21(perm, (floor + Vec2::ONE).as_ivec2()) as usize,
        Vec2::new(dx_ext + SQUISH_CONSTANT_2D, dy_ext + SQUISH_CONSTANT_2D),
    );
    (attn0 + attn1 + attn2) * NORM_CONSTANT_2D
}

fn revert_scale_factor(octaves: u32, persistence: f32) -> f32 {
    let mut gain = 0.0;
    let mut amp = 1.0;
    for _ in 0..octaves {
        gain += amp;
        amp *= persistence;
    }
    if gain == 0.0 {
        1.0
    } else {
        1.0 / gain
    }
}

/// FBM (Fractional Brownian Motion) 叠加多层 OpenSimplex 2D 噪声。
/// `persistence` 控制每层振幅衰减，`lacunarity` 控制频率增长。
pub fn open_simplex_2d_fbm(
    point: Vec2,
    perm: &[u32; TABLE_SIZE],
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
) -> f32 {
    let scale = revert_scale_factor(octaves, persistence);
    let mut value = 0.0;
    let mut amp = 1.0;
    let mut freq = frequency;
    for _ in 0..octaves {
        value += open_simplex_2d(point * freq, perm) * amp;
        amp *= persistence;
        freq *= lacunarity;
    }
    value * scale
}
/// 带种子的 FBM 快捷方式。给定 seed 自动生成置换表并调用 `open_simplex_2d_fbm`。
pub fn open_simplex_2d_fbm_with_seed(
    point: Vec2,
    seed: u32,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
) -> f32 {
    let perm = generate_permutation_table(seed);
    open_simplex_2d_fbm(point, &perm, octaves, frequency, lacunarity, persistence)
}

/// 世界坐标 (x,z) 处的地表高度
pub fn height_at(x: f32, z: f32) -> f32 {
    let h1 = open_simplex_2d_fbm_with_seed(Vec2::new(x, z), 42, 3, 0.02, 2.0, 0.5) * 20.0;
    let h2 = open_simplex_2d_fbm_with_seed(Vec2::new(x, z), 137, 3, 0.08, 2.0, 0.5) * 5.0;
    let h3 = open_simplex_2d_fbm_with_seed(Vec2::new(x, z), 251, 3, 0.25, 2.0, 0.5) * 1.0;
    h1 + h2 + h3
}

/// 密度场：正值 = air，负值 = solid
pub fn density_at(x: f32, y: f32, z: f32) -> f32 {
    y - height_at(x, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_sign_at_surface() {
        let h = height_at(100.0, 200.0);
        let below = density_at(100.0, h - 1.0, 200.0);
        let above = density_at(100.0, h + 1.0, 200.0);
        assert!(below < 0.0, "below surface should be solid");
        assert!(above > 0.0, "above surface should be air");
    }
}
