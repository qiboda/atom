//! Indirect terrain render shader — PBR-like diffuse lighting.
//! Vertex format matches TerrainChunkVertex (32 bytes):
//!   @location(0) position: vec3<f32> (offset 0)
//!   @location(1) normal:   vec3<f32> (offset 16)

@group(0) @binding(0) var<uniform> clip_from_world: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
};

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = clip_from_world * vec4<f32>(in.position, 1.0);
    out.world_normal = in.normal;
    return out;
}

// Simple ambient + directional diffuse (solid), or green (wireframe overlay)
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
#ifdef WIREFRAME
    return vec4<f32>(0.3, 0.9, 0.4, 1.0);
#else
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let n = normalize(in.world_normal);
    let diffuse = max(dot(n, light_dir), 0.0) * 0.7 + 0.3;
    let base = vec3<f32>(0.7, 0.75, 0.8);
    return vec4<f32>(base * diffuse, 1.0);
#endif
}
