
// group 0 is mesh, so can use mesh_vertex_output
// group 1 is material, because only one group, so can not support multipe materials on on mesh.
// group 2 is mesh animation


@group(1) @binding(0)
var<uniform> color: vec4<f32>;

@group(1) @binding(1)
var<uniform> normal: vec3<f32>;

@group(1) @binding(2)
var color_texture: texture_2d<f32>;

@group(1) @binding(3)
var color_sampler: sampler;

@fragment
fn fragment(
    #import bevy_pbr::mesh_vertex_output
) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
