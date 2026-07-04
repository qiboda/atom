//! Indirect terrain render shader — PBR-like diffuse lighting.
//! Vertex format matches TerrainChunkVertex (32 bytes):
//!   @location(0) position: vec3<f32> (offset 0) — world space
//!   @location(1) normal:   vec3<f32> (offset 16)

@group(0) @binding(0) var<uniform> clip_from_world: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
};
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_pos: vec3<f32>,
};

fn chunk_tint(pos: vec3<f32>) -> vec3<f32> {
    let spacing = 15.0;
    let cx = pos.x / spacing;
    let cy = pos.y / spacing;
    let cz = pos.z / spacing;
    let r = select(0.5, 0.8, i32(abs(cx)) % 2 == 0);
    let g = select(0.5, 0.8, i32(abs(cy)) % 2 == 0);
    let b = select(0.5, 0.8, i32(abs(cz)) % 2 == 0);
    return vec3<f32>(r, g, b);
}

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = clip_from_world * vec4<f32>(in.position, 1.0);
    out.world_normal = in.normal;
    out.world_pos = in.position;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
#ifdef WIREFRAME
    return vec4<f32>(0.3, 0.9, 0.4, 1.0);
#else
    let light_dir = normalize(vec3<f32>(0.5, 1.0, 0.3));
    let n = normalize(in.world_normal);
    let diffuse = max(dot(n, light_dir), 0.0) * 0.7 + 0.3;
    let base = vec3<f32>(0.7, 0.75, 0.8);
#ifdef TINT_CHUNK
    let tint = chunk_tint(in.world_pos);
    let blended = mix(base, tint, 0.35);
#else
    let blended = base;
#endif
    return vec4<f32>(blended * diffuse, 1.0);
#endif
}