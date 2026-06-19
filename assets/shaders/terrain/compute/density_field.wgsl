/// 密度场计算。
///
/// 使用 biome 纹理驱动的程序化地形生成：
/// 1. 世界坐标 → UV（terrain_size 来自 TerrainChunkInfo.w）
/// 2. 2×2 邻域采样 biome 类型（Luma8 → ×255 恢复 u8）
/// 3. 双线性混合四个角的 `location.y - biome_height(biome, noise)`
///
/// biome_height：6 种 biome（海洋/森林/沙漠/平原/山地/沼泽），各有不同高度函数。
/// get_biome_type_by_location：单点 biome 查询（用于顶点材质属性）。
#define_import_path terrain::density_field

// #import terrain::csg::csg_utils::apply_csg_operations
#import terrain::main_mesh_bind_group::{ 
    map_config,
    biome_map_texture,
    biome_map_sampler,
    terrain_chunk_info
}

#import noise::open_simplex_seed::{ open_simplex_3d_with_seed }
#import noise::fbm::{ open_simplex_3d_fbm_with_seed, open_simplex_2d_fbm_with_seed}
// #import terrain::biome::{ TerrainType_Underground, TerrainType_Ocean }

fn plane(location: vec3f, normal: vec3f, height: f32) -> f32 {
    // n must be normalized
    return dot(location, normal) + height;
}

fn cube(position: vec3f, half_size: vec3f) -> f32 {
    let q = abs(position) - half_size;
    return length(max(q, vec3f(0.0, 0.0, 0.0))) + min(max(max(q.x, q.y), q.z), 0.0);
}

// x axis from max to 0
fn stairs(position: vec3f, step_height: f32, step_depth: f32, width: f32, num_steps: u32) -> f32 {
    // Loop over each step and accumulate the distance
    var min_dist = 1e6; // Large number to represent infinity
    for (var i = 0u; i < num_steps; i++) {
        // Shift the position down and back for each step
        let step_pos = position - vec3f(0.0, f32(i) * step_height, f32(i) * step_depth);
        
        // Define each step as a box with some width, height, and depth
        let step_dist = cube(step_pos, vec3f(width, step_height, step_depth)); // (Width, Height, Depth of step)
        
        // Find the minimum distance to any step
        min_dist = min(min_dist, step_dist);
    }
    
    return min_dist; // The distance to the nearest step in the stair
}

// SDF function for stair-like sloped shape (Y axis going upwards)
fn sdf_stair_slope(p: vec3<f32>) -> f32 {
    // Fixed stair parameters
    let step_height: f32 = 10.0; // Height of each stair step
    let step_depth: f32 = 30.0;  // Depth of each stair step (horizontal spacing)
    let width: f32 = 10.0;       // Width of the stairs
    let slope: f32 = 0.0;       // Slope of the stair-like structure

    // Calculate the stair pattern by applying a repeating mod along the Z-axis (depth)
    let stair_pattern = p.z % step_depth - slope * p.z;

    // Apply the stair pattern to the Y coordinate to simulate the rise of the stairs
    let sloped_p = vec3<f32>(p.x, p.y - stair_pattern * (step_height / step_depth), p.z);

    // Define the box representing each step with a fixed width
    let step_box_size = vec3<f32>(width * 0.5, step_height * 0.5, step_depth * 0.5);

    // Calculate the SDF for the stair-like sloped structure
    return cube(sloped_p, step_box_size);
}


/// 根据 Rust BiomeType (0-5) 返回 biome 高度
/// 0=海洋, 1=森林, 2=沙漠, 3=平原, 4=山地, 5=沼泽
fn biome_height(biome_type: f32, noise_val: f32) -> f32 {
    if biome_type == 0.0 {
        return -3.0;                          // 海洋 — 深海沟
    } else if biome_type == 1.0 {
        return 4.0 + noise_val * 10.0;        // 森林 — 起伏群山
    } else if biome_type == 2.0 {
        return 2.0 + abs(noise_val) * 12.0;   // 沙漠 — 高沙丘
    } else if biome_type == 3.0 {
        return 3.0 + noise_val * 8.0;         // 平原 — 丘陵
    } else if biome_type == 4.0 {
        return 6.0 + noise_val * 35.0;        // 山地 — 险峰
    } else if biome_type == 5.0 {
        return 1.0 + noise_val * 5.0;         // 沼泽 — 低湿
    }
    return 0.0;
}
/// 根据世界坐标和 biome 纹理计算密度场值
/// 对 biome 纹理 2x2 邻域采样，双线性混合密度值
fn get_terrain_noise(location: vec3f) -> f32 {
    let terrain_size = terrain_chunk_info.chunk_min_location_size.w;
    let terrain_uv = (location.xz + terrain_size * 0.5) / terrain_size;
    let biome_tex_size = vec2f(textureDimensions(biome_map_texture));
    // 2x2 邻域中心的 texel 坐标
    let tex_center = terrain_uv * biome_tex_size - 0.5;
    let base = floor(tex_center);
    let frac = fract(tex_center);
    let max_coord = biome_tex_size - 1.0;

    // 四个角的 texel 坐标，钳位防止越界
    let t00 = vec2u(clamp(base + vec2f(0.0, 0.0), vec2f(0.0), max_coord));
    let t01 = vec2u(clamp(base + vec2f(0.0, 1.0), vec2f(0.0), max_coord));
    let t10 = vec2u(clamp(base + vec2f(1.0, 0.0), vec2f(0.0), max_coord));
    let t11 = vec2u(clamp(base + vec2f(1.0, 1.0), vec2f(0.0), max_coord));

    // 从纹理中采样四个角的 biome 类型
    // Luma8 在 GPU 上归一化为 [0, 1]，乘以 255 恢复原始 u8 值
    let b00 = textureLoad(biome_map_texture, t00, 0).x * 255.0;
    let b01 = textureLoad(biome_map_texture, t01, 0).x * 255.0;
    let b10 = textureLoad(biome_map_texture, t10, 0).x * 255.0;
    let b11 = textureLoad(biome_map_texture, t11, 0).x * 255.0;

    // 世界坐标处的噪声值（四个采样点共享）
    let noise_val = open_simplex_3d_with_seed(location, 232u);

    // 对四个角分别计算密度值（高度 - biome_height）
    let d00 = location.y - biome_height(b00, noise_val);
    let d01 = location.y - biome_height(b01, noise_val);
    let d10 = location.y - biome_height(b10, noise_val);
    let d11 = location.y - biome_height(b11, noise_val);

    // 双线性混合四个角的密度值
    let d0 = mix(d00, d01, frac.y);
    let d1 = mix(d10, d11, frac.y);
    return mix(d0, d1, frac.x);
}

// void value( float v00, float v01, float v10, float v11, vec2 u, out float t, out vec2 grad ) {  
//     float a = v01 - v00;
//     float b = v10 - v00;
//     float c = v11 + v00 - v01 - v10;
    
//     t = v00 + a * u.x + b * u.y + c * u.x * u.y;
    
//     grad.x = a + c * u.y;
//     grad.y = b + c * u.x;
// }


/// 根据世界坐标采样 biome 纹理，返回 biome 类型
fn get_biome_type_by_location(location: vec3f) -> f32 {
    let terrain_size = terrain_chunk_info.chunk_min_location_size.w;
    let terrain_uv = (location.xz + terrain_size * 0.5) / terrain_size;
    let biome_tex_size = vec2f(textureDimensions(biome_map_texture));
    let tex_coord = clamp(terrain_uv * biome_tex_size, vec2f(0.0), biome_tex_size - 1.0);
    return textureLoad(biome_map_texture, vec2u(tex_coord), 0).x * 255.0;
}