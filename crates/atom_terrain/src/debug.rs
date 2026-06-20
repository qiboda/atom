//! 地形调试开关。
//!
//! `TerrainDebugConfig` 控制 wireframe、法线显示等调试渲染选项。

use bevy::prelude::*;
use bevy::pbr::wireframe::{Wireframe, WireframeColor};

use crate::chunk::TerrainChunk;

/// 地形调试配置
#[derive(Resource, Clone, Debug)]
pub struct TerrainDebugConfig {
    /// 是否启用线框渲染
    pub wireframe: bool,
}

impl Default for TerrainDebugConfig {
    fn default() -> Self {
        Self { wireframe: false }
    }
}

/// 当 `TerrainDebugConfig.wireframe` 为 true 时，自动给所有 terrain mesh 子实体添加 Wireframe 组件。
pub fn apply_debug_wireframe(
    debug_config: Res<TerrainDebugConfig>,
    chunks: Query<&Children, With<TerrainChunk>>,
    children: Query<Entity, With<Mesh3d>>,
    mut commands: Commands,
) {
    if !debug_config.wireframe {
        return;
    }
    for chunk_children in chunks.iter() {
        for child in chunk_children.iter() {
            if children.contains(child) {
                commands.entity(child).insert((
                    Wireframe,
                    WireframeColor {
                        color: Color::srgba(0.3, 0.9, 0.4, 1.0),
                    },
                ));
            }
        }
    }
}
