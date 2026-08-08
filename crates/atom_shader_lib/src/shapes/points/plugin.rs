use bevy::{
    asset::{Handle, load_internal_asset, uuid_handle},
    prelude::{MaterialPlugin, Plugin, Shader},
};

use crate::shapes::points::material::PointsMaterial;

/// 点精灵 shader 的内部资源句柄（由 `load_internal_asset!` 注册的固定 UUID）。
pub const POINT_SHADER_HANDLE: Handle<Shader> =
    uuid_handle!("46ed26a8-5848-452a-a12f-bb719fbf4c0d");

/// 点精灵图元插件：加载内部点 shader 并注册 [`PointsMaterial`]。
pub struct PointsPlugin;

impl Plugin for PointsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        load_internal_asset!(
            app,
            POINT_SHADER_HANDLE,
            "shaders/point.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<PointsMaterial>::default());
    }
}
