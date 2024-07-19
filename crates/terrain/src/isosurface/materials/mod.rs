//! 使用地形纹理数组
//! 根据地形的block type，选择不同的纹理
//! 过渡有根据高度混合和权重mix混合。
//! 还有根据自定义纹理进行混合的。纹理内存储的是数组纹理的索引，而不是纹理本身。
use bevy::{
    app::Plugin,
    asset::{load_internal_asset, Handle},
    pbr::MaterialPlugin,
    render::render_resource::Shader,
};
use terrain_mat::TerrainMaterial;

pub mod terrain;
pub mod terrain_mat;

#[derive(Debug, Default)]
pub struct TerrainMaterialPlugin;

pub const TRIPLANAR_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(206437786527492349469145395852868245199);
pub const BIPLANAR_HANDLE: Handle<Shader> =
    Handle::weak_from_u128(32249701768478216984066711652293638536);

impl Plugin for TerrainMaterialPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            TRIPLANAR_HANDLE,
            "shaders/triplanar.wgsl",
            Shader::from_wgsl
        );
        load_internal_asset!(
            app,
            BIPLANAR_HANDLE,
            "shaders/biplanar.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<TerrainMaterial>::default());
    }
}
