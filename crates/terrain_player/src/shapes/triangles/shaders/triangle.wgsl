#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_bindings::mesh
#import bevy_pbr::mesh_functions

struct TriangleMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: TriangleMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let model = mesh_functions::get_model_matrix(vertex.instance_index);
    let world = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
//    let world = vec4<f32>(vertex.position, 1.0);
    out.clip_position = view.view_proj * world;

    return out;
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return material.color;
}