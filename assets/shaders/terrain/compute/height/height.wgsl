/// 生成高度图

#import terrain::biome::get_terrain_noise

struct TerrainMapHeightInfo {
    terrain_size: f32,
}

@group(0) @binding(0)
var<uniform> terrain_map_info: TerrainMapHeightInfo;

// fixed size 1k texture
@group(0) @binding(1) 
var biome_blend_array_texture: texture_2d_array<f32>;
@group(0) @binding(2) 
var biome_blend_array_texture_sampler: sampler;

// range is [-1, 1]
@group(0) @binding(3) 
var height_storage_texture: texture_storage_2d<r32float, write>;


// 8K texture : 8192 x 8192 
// 8192 / 16 = 512, but max workgroup size is 256, so compute 4 pixel(2x2) on each workgroup
@compute @workgroup_size(16, 16, 1)
fn compute_terrain_height(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let compressed_biome_blend_num = textureNumLayers(biome_blend_array_texture);

    // let height_texture_size = vec2f(textureDimensions(height_storage_texture));
    let height_texture_size = vec2f(8192, 8192);
    // let biome_blend_texture_size = vec2f(textureDimensions(biome_blend_array_texture));

    compute_terrain_height_one_pixel(vec2u(invocation_id.x * 2, invocation_id.y * 2), height_texture_size, compressed_biome_blend_num);
    compute_terrain_height_one_pixel(vec2u(invocation_id.x * 2 + 1, invocation_id.y * 2), height_texture_size, compressed_biome_blend_num);
    compute_terrain_height_one_pixel(vec2u(invocation_id.x * 2, invocation_id.y * 2 + 1), height_texture_size, compressed_biome_blend_num);
    compute_terrain_height_one_pixel(vec2u(invocation_id.x * 2 + 1, invocation_id.y * 2 + 1), height_texture_size, compressed_biome_blend_num);
}

fn compute_terrain_height_one_pixel(target_pixel_pos: vec2u, height_texture_size: vec2f, compressed_biome_blend_num: u32) {
    let uv = vec2f(target_pixel_pos) / height_texture_size;
    let location = vec2f(terrain_map_info.terrain_size) * uv - vec2f(terrain_map_info.terrain_size) * 0.5;

    var final_height = 0.0;
    for (var i = 0u; i < compressed_biome_blend_num; i++) {
        let biome_percent = textureSampleLevel(biome_blend_array_texture, biome_blend_array_texture_sampler, uv, i, 0.0);

        if biome_percent.x > 0.0 {
            let noise_x = get_terrain_noise(location, i * 4u);
            final_height += noise_x * biome_percent.x;
        }
        if biome_percent.y > 0.0 {
            let noise_y = get_terrain_noise(location, i * 4u + 1u);
            final_height += noise_y * biome_percent.y;
        }
        if biome_percent.z > 0.0 {
            let noise_z = get_terrain_noise(location, i * 4u + 2u);
            final_height += noise_z * biome_percent.z;
        }
        if biome_percent.w > 0.0 {
            let noise_w = get_terrain_noise(location, i * 4u + 3u);
            final_height += noise_w * biome_percent.w;
        }
    }

    textureStore(height_storage_texture, target_pixel_pos, vec4f(final_height));
}
