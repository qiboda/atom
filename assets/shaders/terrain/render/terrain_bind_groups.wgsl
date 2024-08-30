#define_import_path terrain::terrain_bind_groups

#import terrain::terrain_type::{TerrainMaterial}

@group(2) @binding(0)
var<uniform> terrain_material: TerrainMaterial;

@group(2) @binding(1)
var base_color_texture: texture_2d_array<f32>;
@group(2) @binding(2)
var base_color_sampler: sampler;

@group(2) @binding(3)
var normal_map_texture: texture_2d_array<f32>;
@group(2) @binding(4)
var normal_map_sampler: sampler;

@group(2) @binding(5)
var metallic_roughness_texture: texture_2d_array<f32>;
@group(2) @binding(6)
var metallic_roughness_sampler: sampler;

@group(2) @binding(7)
var occlusion_texture: texture_2d_array<f32>;
@group(2) @binding(8)
var occlusion_sampler: sampler;
