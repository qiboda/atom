#define_import_path terrain::density_field
#import noise::fbm::open_simplex_2d_fbm_with_seed

#import terrain::csg::csg_utils::apply_csg_operations
#import terrain::main_mesh_bind_group::{ csg_operations, map_config, height_map_texture, height_map_sampler, biome_map_sampler, biome_map_texture, terrain_chunk_info }

#import noise::open_simplex_seed::open_simplex_2d_with_seed
#import terrain::biome::{ TerrainType_Underground, TerrainType_Ocean }

fn plane(location: vec3f, normal: vec3f, height: f32) -> f32 {
    // n must be normalized
    return dot(location, normal) + height;
}

fn cube(position: vec3f, half_size: vec3f) -> f32 {
    let q = abs(position) - half_size;
    return length(max(q, vec3f(0.0, 0.0, 0.0))) + min(max(max(q.x, q.y), q.z), 0.0);
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

    let density_value = location.y - height;

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