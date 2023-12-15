#import bevy_pbr::mesh_view_bindings::view
#import bevy_pbr::mesh_bindings::mesh

struct TriangleMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: TriangleMaterial;

fn mesh_position_local_to_world(model: mat3x4<f32>, vertex_position: vec3<f32>) -> vec4<f32> {
    return model * vertex_position;
}

struct Vertex {
    @location(0) position: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;

//    let world = mesh_position_local_to_world(mesh[0].model, vertex.position);
    let world = vec4<f32>(vertex.position, 1.0);
    out.clip_position = view.view_proj * world;

    return out;
}

@fragment
fn fragment() -> @location(0) vec4<f32> {
    return material.color;
}