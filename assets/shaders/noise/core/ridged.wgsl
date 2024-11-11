#define_import_path noise::ridged

// 将ridged的输出值范围重新映射到[-1, 1]
/// required: octaves > 0
fn get_revert_scale_factor(
    octaves: u32,
    persistence: f32, 
    attenuation: f32,
) -> f32 {
    var factor = 0.0;

    var amplitude = 1.0;
    var weight = 1.0;
    var signal = weight * amplitude;

    factor += signal;

    for (var i = 1u; i <= octaves; i++) {
        weight = clamp(signal / pow(attenuation, i), 0.0, 1.0);

        amplitude *= persistence;
        signal = weight * amplitude;

        factor += signal;
    }
    return 2.0 / factor;
}

/// octaves：重复noise次数
/// frequency: 初始的输入坐标重复次数(输入值的缩放)。
/// lacunarity: 每次octaves时，frequency(输入值)的缩放。
/// attenuation: 每次octaves时，应用于weight。
/// persistence: 每次octaves时，noise的输出值的缩放。
///
/// out range，依赖于octaves和persistence, attenuation参数，
/// range的还原见：open_simplex_2d_ridged
fn open_simplex_2d_ridged_scaled(
    point: vec2f,
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
    attenuation: f32,
) -> f32 {
    var value = 0.0;

    var scaled_point = point * frequency;
    var weight = 1.0;
    var scaled_output = 1.0;

    for (var i = 0u; i < octaves; i++) {
        var octaves_value = open_simplex_2d(scaled_point, permutation_table);
        octaves_value = 1.0 - abs(octaves_value);
        octaves_value *= octaves_value;

        octaves_value *= weight;

        weight = octaves_value / attenuation;
        weight = clamp(weight, 0.0, 1.0);

        octaves_value *= scaled_output;

        value += octaves_value;

        scaled_output *= persistence;
        scaled_point *= lacunarity;
    }

    return value;
}

/// 参数说明见：open_simplex_2d_ridged_scaled
/// range: [-1, 1]
fn open_simplex_2d_ridged(
    point: vec2f,
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32, attenuation: f32,
) {
    let revert_factor = get_revert_scale_factor(octaves, persistence, attenuation);
    // range is [0.0, 2.0]
    let ridged_value = open_simplex_2d_ridged_scaled(point, permutation_table, octaves, frequency, lacunarity, persistence) * revert_factor;
    return ridged_value - 1.0;
}

/// octaves：重复noise次数
/// frequency: 初始的输入坐标重复次数(输入值的缩放)。
/// lacunarity: 每次octaves时，frequency(输入值)的缩放。
/// attenuation: 每次octaves时，应用于weight。
/// persistence: 每次octaves时，noise的输出值的缩放。
///
/// out range，依赖于octaves和persistence, attenuation参数，
/// range的还原见：open_simplex_3d_ridged
fn open_simplex_3d_ridged_scaled(
    point: vec3f,
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
    attenuation: f32,
) -> f32 {
    var value = 0.0;

    var scaled_point = point * frequency;
    var weight = 1.0;
    var scaled_output = 1.0;

    for (var i = 0u; i < octaves; i++) {
        var octaves_value = open_simplex_3d(scaled_point, permutation_table);
        octaves_value = 1.0 - abs(octaves_value);
        octaves_value *= octaves_value;

        octaves_value *= weight;

        weight = octaves_value / attenuation;
        weight = clamp(weight, 0.0, 1.0);

        octaves_value *= scaled_output;

        value += octaves_value;

        scaled_output *= persistence;
        scaled_point *= lacunarity;
    }

    return value;
}

/// 参数说明见：open_simplex_2d_ridged_scaled
/// range: [-1, 1]
fn open_simplex_3d_ridged(
    point: vec3f,
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32, attenuation: f32,
) {
    let revert_factor = get_revert_scale_factor(octaves, persistence, attenuation);
    // range is [0.0, 2.0]
    let ridged_value = open_simplex_3d_ridged_scaled(point, permutation_table, octaves, frequency, lacunarity, persistence) * revert_factor;
    return ridged_value - 1.0;
}

/// octaves：重复noise次数
/// frequency: 初始的输入坐标重复次数(输入值的缩放)。
/// lacunarity: 每次octaves时，frequency(输入值)的缩放。
/// attenuation: 每次octaves时，应用于weight。
/// persistence: 每次octaves时，noise的输出值的缩放。
///
/// out range，依赖于octaves和persistence, attenuation参数，
/// range的还原见：open_simplex_2d_ridged
fn open_simplex_4d_ridged_scaled(
    point: vec4f,
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
    attenuation: f32,
) -> f32 {
    var value = 0.0;

    var scaled_point = point * frequency;
    var weight = 1.0;
    var scaled_output = 1.0;

    for (var i = 0u; i < octaves; i++) {
        var octaves_value = open_simplex_4d(scaled_point, permutation_table);
        octaves_value = 1.0 - abs(octaves_value);
        octaves_value *= octaves_value;

        octaves_value *= weight;

        weight = octaves_value / attenuation;
        weight = clamp(weight, 0.0, 1.0);

        octaves_value *= scaled_output;

        value += octaves_value;

        scaled_output *= persistence;
        scaled_point *= lacunarity;
    }

    return value;
}

/// 参数说明见：open_simplex_4d_ridged_scaled
/// range: [-1, 1]
fn open_simplex_4d_ridged(
    point: vec4f,
    permutation_table: ptr<function, array<u32, TABLE_SIZE>>,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32, attenuation: f32,
) {
    let revert_factor = get_revert_scale_factor(octaves, persistence, attenuation);
    // range is [0.0, 2.0]
    let ridged_value = open_simplex_4d_ridged_scaled(point, permutation_table, octaves, frequency, lacunarity, persistence) * revert_factor;
    return ridged_value - 1.0;
}

fn open_simplex_2d_ridged_with_seed(
    point: vec2f,
    seed: u32,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
    attenuation: f32,
) -> f32 {
    let permutation_table = generate_permutation_table(seed);
    return open_simplex_2d_ridged(point, permutation_table, octaves, frequency, lacunarity, persistence, attenuation);
}

fn open_simplex_3d_ridged_with_seed(
    point: vec3f,
    seed: u32,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
    attenuation: f32,
) -> f32 {
    let permutation_table = generate_permutation_table(seed);
    return open_simplex_3d_ridged(point, permutation_table, octaves, frequency, lacunarity, persistence, attenuation);
}

fn open_simplex_4d_ridged_with_seed(
    point: vec4f,
    seed: u32,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
    attenuation: f32,
) -> f32 {
    let permutation_table = generate_permutation_table(seed);
    return open_simplex_4d_ridged(point, permutation_table, octaves, frequency, lacunarity, persistence, attenuation);
}

fn open_simplex_2d_ridged_scaled_with_seed(
    point: vec2f,
    seed: u32,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32,
    attenuation: f32,
) -> f32 {
    let permutation_table = generate_permutation_table(seed);
    return open_simplex_2d_ridged_scaled(point, permutation_table, octaves, frequency, lacunarity, persistence, attenuation);
}

fn open_simplex_3d_ridged_scaled_with_seed(
    point: vec3f,
    seed: u32,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32
    attenuation: f32,
) -> f32 {
    let permutation_table = generate_permutation_table(seed);
    return open_simplex_3d_ridged_scaled(point, permutation_table, octaves, frequency, lacunarity, persistence, attenuation);
}

fn open_simplex_4d_ridged_scaled_with_seed(
    point: vec4f,
    seed: u32,
    octaves: u32,
    frequency: f32,
    lacunarity: f32,
    persistence: f32
    attenuation: f32,
) -> f32 {
    let permutation_table = generate_permutation_table(seed);
    return open_simplex_4d_ridged_scaled(point, permutation_table, octaves, frequency, lacunarity, persistence, attenuation);
}