#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_bindings::mesh

struct LineMaterial {
    line_size: f32,
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: LineMaterial;

struct VertexInput {
    @location(0) position: vec3<f32>,
// #ifdef VERTEX_COLORS
    @location(1) color: vec4<f32>,
// #endif
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
// #ifdef VERTEX_COLORS
    @location(0) color: vec4<f32>,
// #endif
};

@vertex
fn vertex(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
// #ifdef VERTEX_COLORS
    out.color = vertex.color;
// #endif
    return out;
}

struct FragmentInput {
    @location(0) uv: vec2<f32>,
// #ifdef VERTEX_COLORS
    @location(1) color: vec4<f32>,
// #endif
};

@fragment
fn fragment(input: FragmentInput) -> @location(0) vec4<f32> {
// #ifdef VERTEX_COLORS
    // return material.color * input.color;
// else 
    return material.color;
// #endif
}