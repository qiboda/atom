use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

#[derive(AsBindGroup, Clone, Default, Asset, Reflect)]
pub struct TerrainMaterial {
    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,

    #[texture(3)]
    #[sampler(4)]
    pub normal_texture: Option<Handle<Image>>,

    #[texture(5)]
    #[sampler(6)]
    pub metallic_texture: Option<Handle<Image>>,

    #[texture(7)]
    #[sampler(8)]
    pub roughness_texture: Option<Handle<Image>>,
}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shader/terrain/terrain_material.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shader/terrain/terrain_material.wgsl".into()
    }
}
