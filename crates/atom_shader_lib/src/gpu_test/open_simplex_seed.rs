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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permutation_table_length() {
        let table = generate_permutation_table(42);
        assert_eq!(table.len(), TABLE_SIZE);
    }

    #[test]
    fn test_permutation_table_range() {
        let table = generate_permutation_table(42);
        for &val in &table {
            assert!(val < 256, "value {} out of range", val);
        }
    }

    #[test]
    fn test_permutation_table_deterministic() {
        let t1 = generate_permutation_table(42);
        let t2 = generate_permutation_table(42);
        assert_eq!(t1, t2);
    }

    #[test]
    fn test_open_simplex_deterministic() {
        let p = Vec2::new(1.5, 2.5);
        let a = open_simplex_2d_with_seed(p, 42);
        let b = open_simplex_2d_with_seed(p, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn test_open_simplex_range() {
        // Simplex noise should be in roughly [-1, 1] range
        for i in 0..100 {
            let x = i as f32 * 0.1;
            let y = i as f32 * 0.13;
            let val = open_simplex_2d_with_seed(Vec2::new(x, y), 42);
            assert!(
                (-2.0..=2.0).contains(&val),
                "noise {} out of expected range at ({}, {})",
                val,
                x,
                y
            );
        }
    }
}
