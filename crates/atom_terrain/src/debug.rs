//! 地形调试开关。
//!
//! `TerrainDebugConfig` 控制 wireframe、双面渲染等调试选项。
//! 快捷键：
//!   F1 = wireframe, F2 = double_sided, F3 = chunk bounds tint, F4 = world axes
//!
//! Debug 选项通过 indirect draw render pipeline 直接在 GPU 端生效，
//! 不再依赖 Bevy Mesh/Wireframe 组件。

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

/// 地形调试配置
#[derive(Resource, Clone, Debug, ExtractResource)]
pub struct TerrainDebugConfig {
    /// 是否启用线框渲染
    pub wireframe: bool,
    /// 是否双面渲染（关闭背面剔除）
    pub double_sided: bool,
    /// 是否显示 chunk 边界（mesh 着色模式）
    pub show_chunk_bounds: bool,
    /// 是否显示世界坐标轴
    pub show_world_axes: bool,
}

impl Default for TerrainDebugConfig {
    fn default() -> Self {
        Self {
            wireframe: false,
            double_sided: true,
            show_chunk_bounds: false,
            show_world_axes: true,
        }
    }
}

/// 键盘快捷键切换调试开关。
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
    if keys.just_pressed(KeyCode::F3) {
        config.show_chunk_bounds = !config.show_chunk_bounds;
        info!("show_chunk_bounds: {}", config.show_chunk_bounds);
    }
    if keys.just_pressed(KeyCode::F4) {
        config.show_world_axes = !config.show_world_axes;
        info!("show_world_axes: {}", config.show_world_axes);
    }
}

/// 绘制调试可视化：世界坐标轴（chunk 边界通过 mesh tint 显示）
pub fn draw_debug_gizmos(mut gizmos: Gizmos, config: Res<TerrainDebugConfig>) {
    if config.show_world_axes {
        let origin = Vec3::ZERO;
        let len = 100.0;
        gizmos.line(
            origin,
            Vec3::new(len, 0.0, 0.0),
            Srgba::new(1.0, 0.2, 0.2, 1.0),
        ); // X red
        gizmos.line(
            origin,
            Vec3::new(0.0, len, 0.0),
            Srgba::new(0.2, 1.0, 0.2, 1.0),
        ); // Y green
        gizmos.line(
            origin,
            Vec3::new(0.0, 0.0, len),
            Srgba::new(0.2, 0.4, 1.0, 1.0),
        ); // Z blue
    }
}
