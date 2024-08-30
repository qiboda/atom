//! 使用地形纹理数组
//! 根据地形的block type，选择不同的纹理
//! 过渡有根据高度混合和权重mix混合。
//! 还有根据自定义纹理进行混合的。纹理内存储的是数组纹理的索引，而不是纹理本身。
use atom_shader_lib::shaders_plugin;
use bevy::{
    app::Plugin, asset::Handle, pbr::MaterialPlugin, prelude::Resource,
    render::render_resource::Shader,
};
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
