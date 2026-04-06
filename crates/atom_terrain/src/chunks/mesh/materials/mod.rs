use atom_shader_lib::shaders_plugin;
use bevy::{app::Plugin, asset::Handle, pbr::MaterialPlugin, prelude::Resource, shader::Shader};
use terrain_material::TerrainMaterial;

pub mod terrain_material;

#[derive(Debug, Default)]
pub struct TerrainMaterialPlugin;

#[derive(Resource, Default)]
pub struct TerrainMaterialShader {
    pub triplanar: Handle<Shader>,
    pub biplanar: Handle<Shader>,
    pub terrain_material: Handle<Shader>,
}

shaders_plugin!(
    Terrain,
    Material,
    (
        triplanar_shader -> "shaders/terrain/planar/triplanar.wgsl",
        biplanar_shader -> "shaders/terrain/planar/biplanar.wgsl",
        terrain_type_shader -> "shaders/terrain/render/terrain_type.wgsl",
        terrain_bind_groups_shader -> "shaders/terrain/render/terrain_bind_groups.wgsl"
    )
);

impl Plugin for TerrainMaterialPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(TerrainMaterialShadersPlugin)
            .add_plugins(MaterialPlugin::<TerrainMaterial>::default());
    }
}
