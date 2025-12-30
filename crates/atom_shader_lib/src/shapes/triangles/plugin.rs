use bevy::{
    asset::{Handle, load_internal_asset, uuid_handle},
    prelude::{MaterialPlugin, Plugin, Shader},
};

use crate::shapes::triangles::material::TriangleMaterial;

pub const TRIANGLES_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("e9eaa137-68a4-4f56-94fb-cfb92d589c06");

pub struct TrianglesPlugin;

impl Plugin for TrianglesPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            TRIANGLES_SHADER_HANDLE,
            "shaders/triangle.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<TriangleMaterial>::default());
    }
}
