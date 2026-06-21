//! 地形调试开关。
//!
//! `TerrainDebugConfig` 控制 wireframe、双面渲染等调试选项。
//! 快捷键：F1 切 wireframe，F2 切 double_sided。
//!
//! Debug 选项通过 indirect draw render pipeline 直接在 GPU 端生效，
//! 不再依赖 Bevy Mesh/Wireframe 组件。

use bevy::prelude::*;


use bevy::render::extract_resource::ExtractResource;

/// 地形调试配置
#[derive(Resource, Clone, Debug, Default, ExtractResource)]
pub struct TerrainDebugConfig {
    /// 是否启用线框渲染
    pub wireframe: bool,
    /// 是否双面渲染（关闭背面剔除）
    pub double_sided: bool,
}

/// 键盘快捷键切换调试开关。
/// F1 = wireframe，F2 = double_sided。
/// 两者均即时生效（render pipeline 预建 4 种 variant，渲染时按 config 选择）。
pub fn debug_keyboard_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<TerrainDebugConfig>,
) {
    if keys.just_pressed(KeyCode::F1) {
        config.wireframe = !config.wireframe;
        info!("wireframe: {}", config.wireframe);
    }
    if keys.just_pressed(KeyCode::F2) {
        config.double_sided = !config.double_sided;
        info!("double_sided: {}", config.double_sided);
    }
}

/// （Phase 3: indirect draw 管线不经过 Bevy Mesh 组件，wireframe 由 pipeline variant 实现。
///  此函数保留为空，待 Phase 2 彻底移除后删除。)
#[allow(dead_code)]
pub fn apply_debug_wireframe() {}
