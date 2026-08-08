use bevy::{
    asset::{Handle, load_internal_asset, uuid_handle},
    prelude::{MaterialPlugin, Plugin, Shader},
};

use crate::shapes::triangles::material::TriangleMaterial;

/// 三角形 shader 的内部资源句柄（由 `load_internal_asset!` 注册的固定 UUID）。
pub const TRIANGLES_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("e9eaa137-68a4-4f56-94fb-cfb92d589c06");

/// 三角形图元插件：加载内部三角形 shader 并注册 [`TriangleMaterial`]。
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
