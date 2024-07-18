use bevy::{
    asset::{load_internal_asset, Handle},
    prelude::{MaterialPlugin, Plugin, Shader},
};

use crate::shapes::triangles::material::TriangleMaterial;

pub const TRIANGLES_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(76789012569012347213904);

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
