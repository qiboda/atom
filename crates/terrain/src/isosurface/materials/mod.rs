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
