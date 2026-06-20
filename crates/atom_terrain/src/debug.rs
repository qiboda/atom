//! 地形调试开关。
//!
//! `TerrainDebugConfig` 控制 wireframe、双面渲染等调试选项。

use bevy::prelude::*;
use bevy::pbr::wireframe::{Wireframe, WireframeColor};

use crate::chunk::TerrainChunk;
use crate::mesh::GlobalTerrainMesh;

/// 地形调试配置
#[derive(Resource, Clone, Debug)]
pub struct TerrainDebugConfig {
    /// 是否启用线框渲染
    pub wireframe: bool,
    /// 是否双面渲染（关闭背面剔除）
    pub double_sided: bool,
}

impl Default for TerrainDebugConfig {
    fn default() -> Self {
        Self {
            wireframe: false,
            double_sided: false,
        }
    }
}

/// 当 `TerrainDebugConfig.wireframe` 为 true 时，自动给 terrain mesh 添加 Wireframe 组件。
pub fn apply_debug_wireframe(
    debug_config: Res<TerrainDebugConfig>,
    // Phase 2: per-chunk mesh children
    chunks: Query<&Children, With<TerrainChunk>>,
    // Phase 3: global mesh entity
    global_meshes: Query<Entity, With<GlobalTerrainMesh>>,
    children: Query<Entity, With<Mesh3d>>,
    mut commands: Commands,
) {
    if !debug_config.wireframe {
        return;
    }
    // Phase 2: chunk children
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
    // Phase 3: global mesh
    for entity in global_meshes.iter() {
        commands.entity(entity).insert((
            Wireframe,
            WireframeColor {
                color: Color::srgba(0.3, 0.9, 0.4, 1.0),
            },
        ));
    }
}
