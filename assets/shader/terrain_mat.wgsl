
// group 0 is mesh, so can use mesh_vertex_output
// group 1 is material, because only one group, so can not support multipe materials on on mesh.
// group 2 is mesh animation

#import bevy_pbr::mesh_vertex_output MeshVertexOutput


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

@fragment
fn fragment(
    in: MeshVertexOutput
) -> @location(0) vec4<f32> {
    return base_color;
}
