use bevy::math::{UVec4, Vec2};

use super::{open_simplex::open_simplex_2d, xorshift_128::xorshift_128_with_seed};

pub const TABLE_SIZE: usize = 256;

#[allow(dead_code)]
pub fn generate_permutation_table(seed: u32) -> [u32; TABLE_SIZE] {
    let mut permutation_table = [0; TABLE_SIZE];

    let mut seed_vec4: UVec4 = UVec4::new(1, seed, seed, seed);
    (0..TABLE_SIZE).for_each(|i| {
        permutation_table[i] = xorshift_128_with_seed(&mut seed_vec4) % 256;
    });

    permutation_table
}

/// 每次都重新生成了permutation_table，性能不好。
/// 如果重复调用，不应该使用这个函数
#[allow(dead_code)]
pub fn open_simplex_2d_with_seed(point: Vec2, seed: u32) -> f32 {
    let permutation_table = generate_permutation_table(seed);
    open_simplex_2d(point, permutation_table)
}
