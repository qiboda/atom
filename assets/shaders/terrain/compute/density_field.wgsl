#define_import_path terrain::density_field

#import terrain::csg::csg_utils::apply_csg_operations
#import terrain::main_mesh_bind_group::{ csg_operations, map_config, height_map_texture, height_map_sampler, biome_map_sampler, biome_map_texture, terrain_chunk_info }

#import noise::open_simplex_seed::{ open_simplex_3d_with_seed }
#import noise::fbm::{ open_simplex_3d_fbm_with_seed, open_simplex_2d_fbm_with_seed}
#import terrain::biome::{ TerrainType_Underground, TerrainType_Ocean }

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

fn get_terrain_noise(location: vec3f) -> f32 {
    // let density_value = plane(location, vec3f(0.0, 1.0, 1.0), -1.0);
    // let loc = location + vec3f(6.0, 6.0, 6.0);
    // let density_value = cube(loc, vec3f(14.0, 14.0, 14.0));

    // let pos = location.xz / map_config.pixel_size;

    // let height_humidity_temperature = textureSampleLevel(map_height_climate_texture, map_height_climate_sampler, pos, 0.0).xyz;

    let terrain_size = terrain_chunk_info.chunk_min_location_size.w;
    let terrain_uv = (location.xz + terrain_size * 0.5) / terrain_size;
    let height = textureSampleLevel(height_map_texture, height_map_sampler, terrain_uv, 0.0).x * map_config.terrain_height;

    var density_value = location.y - height;

    // let height_comp = textureGather(0, height_map_texture, height_map_sampler, terrain_uv);
    // let max_height = max(max(height_comp.x, height_comp.y), max(height_comp.z, height_comp.w));
    // let min_height = min(min(height_comp.x, height_comp.y), min(height_comp.z, height_comp.w));
    // if max_height - min_height > 0.2 && location.y < max_height * map_config.terrain_height && location.y > min_height * map_config.terrain_height {
    //     // let offset = open_simplex_3d_with_seed(location, 232u) * 2.0;
    //     let offset_x = open_simplex_2d_fbm_with_seed(location.xy, 232u, 1u, 1.0, 0.2, 1.0) * 1.0;
    //     // let offset_z = open_simplex_2d_fbm_with_seed(location.zy, 232u, 1u, 1.0, 0.2, 1.0) * 1.0;
    //     density_value += max(offset_x, 0.0);
    // }

    // return sdf_stair_slope(location);

    return apply_csg_operations(location, density_value);
}

// void value( float v00, float v01, float v10, float v11, vec2 u, out float t, out vec2 grad ) {  
//     float a = v01 - v00;
//     float b = v10 - v00;
//     float c = v11 + v00 - v01 - v10;
    
//     t = v00 + a * u.x + b * u.y + c * u.x * u.y;
    
//     grad.x = a + c * u.y;
//     grad.y = b + c * u.x;
// }


fn get_biome_type_by_location(location: vec3f) -> u32 {
    let terrain_size = terrain_chunk_info.chunk_min_location_size.w;
    let terrain_uv = (location.xz + terrain_size * 0.5) / terrain_size;

    let height = textureSampleLevel(height_map_texture, height_map_sampler, terrain_uv, 0.0).x * map_config.terrain_height;
    let biome_size = vec2f(textureDimensions(biome_map_texture));
    var map_biome = textureLoad(biome_map_texture, vec2u(terrain_uv * biome_size), 0).x;

    // let coord = vec2u(biome_size * terrain_uv);

    // let chunk_size = terrain_chunk_info.voxel_size * f32(terrain_chunk_info.voxel_num);
    // let chunk_uv = (location.xz + vec2f(chunk_size, chunk_size) * 0.5) / chunk_size;

    var biome = select(map_biome, TerrainType_Ocean, map_biome == 255u);
    biome = select(TerrainType_Underground, biome, location.y >= height);
    return biome;
}