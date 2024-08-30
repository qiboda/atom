#define_import_path terrain::terrain_type

struct TerrainVertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) material: u32,
}

struct TerrainVertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) world_position: vec4f,
    @location(1) world_normal: vec3f,
    @location(2) @interpolate(flat) material_weights: vec4f,
    @location(3) @interpolate(flat) instance_index: u32,
}

struct TerrainFragmentOutput {
    @location(0) color: vec4<f32>,
}

struct TerrainMaterial {
    base_color: vec4f,
    lod: u32,
    perceptual_roughness: f32,
    metallic: f32,
    // use standard material flags
    flags: u32,
    reflectance: f32,
    attenuation_distance: f32,
    attenuation_color: vec4f,
}
