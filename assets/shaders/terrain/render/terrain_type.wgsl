#define_import_path terrain::terrain_type

#import terrain::biome::TerrainType_MAX

struct TerrainVertexInput {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3f,
    @location(1) normal: vec3f,
    @location(2) biome: u32,
}

struct TerrainVertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) world_position: vec4f,
    @location(1) world_normal: vec3f,
    @location(2) @interpolate(flat) instance_index: u32,
    @location(3) @interpolate(linear) biome_weights_a: vec4f,
    @location(4) @interpolate(linear) biome_weights_b: vec4f,
    @location(5) @interpolate(linear) biome_weights_c: vec4f,
    @location(6) @interpolate(linear) biome_weights_d: vec4f,
    @location(7) @interpolate(linear) biome_weights_e: vec4f,
    @location(8) @interpolate(linear) biome_weights_f: vec4f,
}

struct TerrainFragmentOutput {
    @location(0) color: vec4<f32>,
}

struct TerrainMaterial {
    lod: u32,
    perceptual_roughness: f32,
    metallic: f32,
    // use standard material flags
    flags: u32,
    reflectance: f32,
    attenuation_distance: f32,
    attenuation_color: vec4f,
    biome_colors: array<TerrainBiomeColor, TerrainType_MAX>,
}


struct TerrainBiomeColor {
    base_color: vec4f,
    biome: u32,
}