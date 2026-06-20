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
/// 根据世界坐标和 biome 纹理计算密度场值
/// 对 biome 纹理 2x2 邻域采样，双线性混合密度值
fn get_terrain_noise(location: vec3f) -> f32 {
    if map_config.use_biome == 0u {
        return get_terrain_noise_no_biome(location);
    }

    let terrain_size = terrain_chunk_info.chunk_min_location_size.w;
    let terrain_uv = (location.xz + terrain_size * 0.5) / terrain_size;
    let biome_tex_size = vec2f(textureDimensions(biome_map_texture));
    // 2x2 邻域中心的 texel 坐标
    let tex_center = terrain_uv * biome_tex_size - 0.5;
    let tex_base = vec2i(tex_center);
    let fx = fract(tex_center.x);
    let fy = fract(tex_center.y);
    let t00 = textureLoad(biome_map_texture, tex_base + vec2i(0, 0), 0).x * 255.0;
    let t10 = textureLoad(biome_map_texture, tex_base + vec2i(1, 0), 0).x * 255.0;
    let t01 = textureLoad(biome_map_texture, tex_base + vec2i(0, 1), 0).x * 255.0;
    let t11 = textureLoad(biome_map_texture, tex_base + vec2i(1, 1), 0).x * 255.0;
    // 2D FBM 噪声（高度图风格，XZ 平面），频率 0.08 在 chunk 尺度可见
    let noise_val = open_simplex_2d_fbm_with_seed(location.xz, 232u, 3u, 0.08, 2.0, 0.5);
    let h00 = biome_height(t00, noise_val);
    let h10 = biome_height(t10, noise_val);
    let h01 = biome_height(t01, noise_val);
    let h11 = biome_height(t11, noise_val);
    // 双线性混合 biome 密度值
    let h0 = mix(h00, h10, fx);
    let h1 = mix(h01, h11, fx);
    let biome_height_val = mix(h0, h1, fy);
    return location.y - biome_height_val;
}

/// 无 biome 纹理的纯噪声密度场
/// 使用 2D FBM 噪声直接生成地形高度
fn get_terrain_noise_no_biome(location: vec3f) -> f32 {
    // 基本地形：多层 FBM 噪声叠加
    let h1 = open_simplex_2d_fbm_with_seed(location.xz, 42u, 3u, 0.02, 2.0, 0.5) * 20.0;
    // 细节噪声
    let h2 = open_simplex_2d_fbm_with_seed(location.xz, 137u, 3u, 0.08, 2.0, 0.5) * 5.0;
    // 微细节
    let h3 = open_simplex_2d_fbm_with_seed(location.xz, 251u, 3u, 0.25, 2.0, 0.5) * 1.0;
    return location.y - (h1 + h2 + h3);
}

/// 单点 biome 查询（用于顶点属性）
fn get_biome_type_by_location(location: vec3f) -> f32 {
    if map_config.use_biome == 0u {
        return 0.0;  // 无 biome 时返回默认值
    }
    // XZ 平面采样 biome 纹理
    let terrain_size = terrain_chunk_info.chunk_min_location_size.w;
    let terrain_uv = (location.xz + terrain_size * 0.5) / terrain_size;
    let biome_tex_size = vec2f(textureDimensions(biome_map_texture));
    let tex_coord = clamp(terrain_uv * biome_tex_size, vec2f(0.0), biome_tex_size - 1.0);
    return textureLoad(biome_map_texture, vec2u(tex_coord), 0).x * 255.0;
}