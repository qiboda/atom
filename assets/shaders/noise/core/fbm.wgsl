/// TODO 使用macro，确定使用的noise的类型。但这样就没办法，同时使用两种fbm了，还是需要再考虑一下。
#define_import_path noise::fbm

#import noise::open_simplex::{TABLE_SIZE, open_simplex_2d, open_simplex_3d, open_simplex_4d}
#import noise::open_simplex_seed::{generate_permutation_table}

// 将fmb的输出值范围重新映射到[-1, 1]
fn get_revert_scale_factor(
    octaves: u32,
    persistence: f32
) -> f32 {
    var factor = 0.0;
    for (var i = 1u; i <= octaves; i++) {
        factor += pow(persistence, f32(i));
    }
    return 1.0 / factor;
}

/// octaves：重复noise次数
/// frequency: 初始的输入坐标重复次数(输入值的缩放)。
/// lacunarity: 每次octaves时，frequency(输入值)的缩放。
/// persistence: 每次octaves时，noise输出值的缩放
///
/// out range，依赖于octaves和persistence参数，
/// 可以使用get_revert_scale_factor函数得到还原因子，乘以还原因子可以让范围到[-1, 1]
fn open_simplex_2d_fbm_scaled(
    point: vec2f,
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32
) -> f32 {
    var value = 0.0;
    var scaled_point = point * frequency;
    var scaled_output = persistence;
    for (var i = 0u; i < octaves; i++) {
        let octaves_value = open_simplex_2d(scaled_point, permutation_table);
        value += octaves_value * scaled_output;

        scaled_output *= persistence;
        scaled_point *= lacunarity;
    }

    return value;
}

/// 参数说明见：open_simplex_2d_fbm_scaled
/// range: [-1, 1]
fn open_simplex_2d_fbm(
    point: vec2f,
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32
) -> f32 {
    let revert_factor = get_revert_scale_factor(octaves, persistence);
    return open_simplex_2d_fbm_scaled(point, permutation_table, octaves, frequency, lacunarity, persistence) * revert_factor;
}

/// octaves：重复noise次数
/// frequency: 初始的输入坐标重复次数(输入值的缩放)。
/// lacunarity: 每次octaves时，frequency(输入值)的缩放。
/// persistence: 每次octaves时，noise输出值的缩放
///
/// out range，依赖于octaves和persistence参数，
/// 可以使用get_revert_scale_factor函数得到还原因子，乘以还原因子可以让范围到[-1, 1]
fn open_simplex_3d_fbm_scaled(
    point: vec3f,
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32
) -> f32 {
    var value = 0.0;
    var scaled_point = point * frequency;
    var scaled_output = persistence;
    for (var i = 0u; i < octaves; i++) {
        let octaves_value = open_simplex_3d(scaled_point, permutation_table);
        value += octaves_value * scaled_output;

        scaled_output *= persistence;
        scaled_point *= lacunarity;
    }

    return value;
}

/// 参数说明见：open_simplex_3d_fbm_scaled
/// range: [-1, 1]
fn open_simplex_3d_fbm(
    point: vec3f,
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32
) -> f32 {
    let revert_factor = get_revert_scale_factor(octaves, persistence);
    return open_simplex_3d_fbm_scaled(point, permutation_table, octaves, frequency, lacunarity, persistence) * revert_factor;
}

/// octaves：重复noise次数
/// frequency: 初始的输入坐标重复次数(输入值的缩放)。
/// lacunarity: 每次octaves时，frequency(输入值)的缩放。
/// persistence: 每次octaves时，noise输出值的缩放
///
/// out range，依赖于octaves和persistence参数，
/// 可以使用get_revert_scale_factor函数得到还原因子，乘以还原因子可以让范围到[-1, 1]
fn open_simplex_4d_fbm_scaled(
    point: vec4f,
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32
) -> f32 {
    var value = 0.0;
    var scaled_point = point * frequency;
    var scaled_output = persistence;
    for (var i = 0u; i < octaves; i++) {
        let octaves_value = open_simplex_4d(scaled_point, permutation_table);
        value += octaves_value * scaled_output;

        scaled_output *= persistence;
        scaled_point *= lacunarity;
    }

    return value;
}

/// 参数说明见：open_simplex_4d_fbm_scaled
/// range: [-1, 1]
fn open_simplex_4d_fbm(
    point: vec4f,
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32
) -> f32 {
    let revert_factor = get_revert_scale_factor(octaves, persistence);
    return open_simplex_4d_fbm_scaled(point, permutation_table, octaves, frequency, lacunarity, persistence) * revert_factor;
}

fn open_simplex_2d_fbm_with_seed(
    point: vec2f,
    seed: u32,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32
) -> f32 {
    var permutation_table = generate_permutation_table(seed);
    return open_simplex_2d_fbm(point, &permutation_table, octaves, frequency, lacunarity, persistence);
}

fn open_simplex_3d_fbm_with_seed(
    point: vec3f,
    seed: u32,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32
) -> f32 {
    var permutation_table = generate_permutation_table(seed);
    return open_simplex_3d_fbm(point, &permutation_table, octaves, frequency, lacunarity, persistence);
}

fn open_simplex_4d_fbm_with_seed(
    point: vec4f,
    seed: u32,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32
) -> f32 {
    var permutation_table = generate_permutation_table(seed);
    return open_simplex_4d_fbm(point, &permutation_table, octaves, frequency, lacunarity, persistence);
}

fn open_simplex_2d_fbm_scaled_with_seed(
    point: vec2f,
    seed: u32,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32
) -> f32 {
    var permutation_table = generate_permutation_table(seed);
    return open_simplex_2d_fbm_scaled(point, &permutation_table, octaves, frequency, lacunarity, persistence);
}

fn open_simplex_3d_fbm_scaled_with_seed(
    point: vec3f,
    seed: u32,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32
) -> f32 {
    var permutation_table = generate_permutation_table(seed);
    return open_simplex_3d_fbm_scaled(point, &permutation_table, octaves, frequency, lacunarity, persistence);
}

fn open_simplex_4d_fbm_scaled_with_seed(
    point: vec4f,
    seed: u32,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32
) -> f32 {
    var permutation_table = generate_permutation_table(seed);
    return open_simplex_4d_fbm_scaled(point, &permutation_table, octaves, frequency, lacunarity, persistence);
}