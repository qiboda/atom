// group 0 is mesh, so can use mesh_vertex_output
// group 1 is material, because only one group, so can not support multipe materials on on mesh.
// group 2 is mesh animation

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput}
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput}
}
#endif

@group(2) @binding(0)
var<uniform> color: vec4<f32>;

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var out: FragmentOutput;
    out.color = color;
    return out;
}
