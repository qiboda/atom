#define_import_path noise::open_simplex_seed

#import noise::open_simplex::{open_simplex_2d, open_simplex_3d, open_simplex_4d, TABLE_SIZE}
#import random::xorshift_128::{xorshift_128_with_seed}

fn generate_permutation_table(seed: u32) -> array<u32, TABLE_SIZE> {
    var permutation_table: array<u32, TABLE_SIZE> = array<u32, TABLE_SIZE>();

    var seed_vec4 = vec4u(1, seed, seed, seed);
    for (var i = 0u; i < TABLE_SIZE; i++) {
        permutation_table[i] = xorshift_128_with_seed(&seed_vec4) % TABLE_SIZE;
    }

    return permutation_table;
}

/// 每次都重新生成了permutation_table，性能不好。
/// 如果重复调用，不应该使用这个函数
fn open_simplex_2d_with_seed(point: vec2f, seed: u32) -> f32 {
    var permutation_table = generate_permutation_table(seed);
    return open_simplex_2d(point, &permutation_table);
}

/// 每次都重新生成了permutation_table，性能不好。
/// 如果重复调用，不应该使用这个函数
fn open_simplex_3d_with_seed(point: vec3f, seed: u32) -> f32 {
    var permutation_table = generate_permutation_table(seed);
    return open_simplex_3d(point, &permutation_table);
}

/// 每次都重新生成了permutation_table，性能不好。
/// 如果重复调用，不应该使用这个函数
fn open_simplex_4d_with_seed(point: vec4f, seed: u32) -> f32 {
    var permutation_table = generate_permutation_table(seed);
    return open_simplex_4d(point, &permutation_table);
}

