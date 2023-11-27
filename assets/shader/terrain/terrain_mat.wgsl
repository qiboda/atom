
// group 0 is mesh, so can use mesh_vertex_output
// group 1 is material, because only one group, so can not support multipe materials on on mesh.
// group 2 is mesh animation

#import bevy_pbr::forward_io::VertexOutput


@group(1) @binding(0)
var<uniform> base_color: vec4<f32>;

@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;

@group(1) @binding(2)
var base_color_texture_sampler: sampler;

@group(1) @binding(3)
var normal_map_texture: texture_2d<f32>;

@group(1) @binding(4)
var normal_map_sampler: sampler;

@group(1) @binding(5)
var metallic_map_texture: texture_2d<f32>;

@group(1) @binding(6)
var metallic_map_sampler: sampler;

@group(1) @binding(7)
var roughness_map_texture: texture_2d<f32>;

@group(1) @binding(8)
var roughness_map_sampler: sampler;

@group(1) @binding(9)
var occlusion_map_texture: texture_2d<f32>;

@group(1) @binding(10)
var occlusion_map_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput
) -> @location(0) vec4<f32> {
//    return base_color;

    var uv = fract(mesh.position.xy / vec2<f32>(16.0, 16.0));
    return textureSample(base_color_texture, base_color_texture_sampler, uv);
}
