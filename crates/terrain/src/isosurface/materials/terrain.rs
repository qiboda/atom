use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
};

#[derive(AsBindGroup, Clone, Default, Asset, Reflect)]
pub struct TerrainMaterial {
    #[uniform(100)]
    pub(crate) base_color: LinearRgba,
}

impl MaterialExtension for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shader/terrain/terrain_mat.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shader/terrain/terrain_mat.wgsl".into()
    }
}

pub type TerrainExtendedMaterial = ExtendedMaterial<StandardMaterial, TerrainMaterial>;
