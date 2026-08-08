use bevy::{
    asset::{load_internal_asset, uuid_handle},
    prelude::*,
};

use crate::shapes::lines::material::LineMaterial;

/// 线段 shader 的内部资源句柄（由 `load_internal_asset!` 注册的固定 UUID）。
pub const LINE_SHADER_HANDLE: Handle<Shader> = uuid_handle!("a0be7962-b573-49d2-aac6-2b72cb62521e");

/// 线段图元插件：加载内部线段 shader 并注册 [`LineMaterial`]。
pub struct LinesPlugin;

impl Plugin for LinesPlugin {
    fn build(&self, app: &mut App) {
        load_internal_asset!(
            app,
            LINE_SHADER_HANDLE,
            "shaders/line.wgsl",
            Shader::from_wgsl
        );

        app.add_plugins(MaterialPlugin::<LineMaterial>::default());
    }
}
