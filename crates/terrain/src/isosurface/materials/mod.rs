//! 使用地形纹理数组
//! 根据地形的block type，选择不同的纹理
//! 过渡有根据高度混合和权重mix混合。
//! 还有根据自定义纹理进行混合的。纹理内存储的是数组纹理的索引，而不是纹理本身。
use bevy::{
    app::Plugin,
    asset::{DirectAssetAccessExt, Handle},
    pbr::MaterialPlugin,
    prelude::Resource,
    render::{render_resource::Shader, RenderApp},
};
use terrain_mat::TerrainMaterial;

pub mod terrain_mat;

#[derive(Debug, Default)]
pub struct TerrainMaterialPlugin;

#[derive(Resource, Default)]
pub struct TerrainMaterialShader {
    pub triplanar: Handle<Shader>,
    pub biplanar: Handle<Shader>,
    pub terrain_material: Handle<Shader>,
}

impl Plugin for TerrainMaterialPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // let world = app.world();
        let render_app = app.get_sub_app(RenderApp).unwrap();
        let render_world = render_app.world();
        app.insert_resource(TerrainMaterialShader {
            triplanar: render_world.load_asset("shaders/terrain/triplanar.wgsl"),
            biplanar: render_world.load_asset("shaders/terrain/biplanar.wgsl"),
            terrain_material: render_world.load_asset("shaders/terrain/terrain_material.wgsl"),
        });

        app.add_plugins(MaterialPlugin::<TerrainMaterial>::default());
    }
}
