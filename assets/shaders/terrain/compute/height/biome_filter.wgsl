#import terrain::map::map_type::TerrainMapInfo

@group(0) @binding(0)
var<uniform> terrain_map_info: TerrainMapInfo;

@group(0) @binding(1) 
var biome_blend_array_texture: texture_2d_array<f32>;
@group(0) @binding(2) 
var output_biome_blend_array_texture: texture_storage_2d_array<rgba8unorm, write>;

// 8K texture : 8192 x 8192 
// 8192 / 16 = 512, but max workgroup size is 256, so compute 4 pixel(2x2) on each workgroup
@compute @workgroup_size(16, 16, 1)
fn compute_terrain_map_biome(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    let compressed_biome_blend_num = textureNumLayers(biome_blend_array_texture);

    for (var i = 0u; i < terrain_map_info.pixel_num_per_kernel; i++) {
        let x = i % 2u;
        let y = i / 2u;
        box_filter(vec2u(invocation_id.x * terrain_map_info.stride + x, invocation_id.y * terrain_map_info.stride + y), compressed_biome_blend_num);
    }
}

fn box_filter(target_pixel_pos: vec2u, compressed_biome_blend_num: u32) {
    let filter_size = 16;

    var colors = array<vec4f, 64>();
    var sum = 0.0;
    for (var index = 0u; index < compressed_biome_blend_num; index++) {
        var color = vec4f(0.0, 0.0, 0.0, 0.0);
        for (var x = -filter_size; x < filter_size; x++) {
            for (var y = -filter_size; y < filter_size; y++) {
                color += textureLoad(biome_blend_array_texture, vec2i(target_pixel_pos) + vec2i(x, y), index, 0);
            }
        }

        colors[index] = color / f32(filter_size * filter_size * 4);
        sum += colors[index].x + colors[index].y + colors[index].z + colors[index].w;
    }

    for (var i = 0u; i < compressed_biome_blend_num; i++) {
        let color = colors[i] / sum;
        textureStore(output_biome_blend_array_texture, target_pixel_pos, i, color);
    }
}
