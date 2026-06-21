//! 地形调试开关。
//!
//! `TerrainDebugConfig` 控制 wireframe、双面渲染等调试选项。
//! 快捷键：F1 切 wireframe，F2 切 double_sided。

use bevy::pbr::wireframe::{Wireframe, WireframeColor};
use bevy::prelude::*;

use crate::chunk::TerrainChunk;
use crate::mesh::GlobalTerrainMesh;

/// 地形调试配置
#[derive(Resource, Clone, Debug, Default)]
pub struct TerrainDebugConfig {
    /// 是否启用线框渲染
    pub wireframe: bool,
    /// 是否双面渲染（关闭背面剔除）
    pub double_sided: bool,
}

/// 键盘快捷键切换调试开关。
/// F1 = wireframe（即时生效），F2 = double_sided（需移动摄像机触发 mesh 重建）。
pub fn debug_keyboard_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<TerrainDebugConfig>,
    // Phase 2 wireframe: chunk mesh entities
    chunks: Query<&Children, With<TerrainChunk>>,
    // Phase 3 wireframe: global mesh entity
    global_meshes: Query<Entity, With<GlobalTerrainMesh>>,
    mesh_children: Query<Entity, With<Mesh3d>>,
    mut observer: ResMut<crate::compute::global_compute::TerrainObserver>,
    mut commands: Commands,
) {
    if keys.just_pressed(KeyCode::F1) {
        config.wireframe = !config.wireframe;
        info!("wireframe: {}", config.wireframe);

        if config.wireframe {
            for chunk_children in chunks.iter() {
                for child in chunk_children.iter() {
                    if mesh_children.contains(child) {
                        commands.entity(child).insert((
                            Wireframe,
                            WireframeColor {
                                color: Color::srgba(0.3, 0.9, 0.4, 1.0),
                            },
                        ));
                    }
                }
            }
            for entity in global_meshes.iter() {
                commands.entity(entity).insert((
                    Wireframe,
                    WireframeColor {
                        color: Color::srgba(0.3, 0.9, 0.4, 1.0),
                    },
                ));
            }
        } else {
            for chunk_children in chunks.iter() {
                for child in chunk_children.iter() {
                    commands
                        .entity(child)
                        .remove::<Wireframe>()
                        .remove::<WireframeColor>();
                }
            }
            for entity in global_meshes.iter() {
                commands
                    .entity(entity)
                    .remove::<Wireframe>()
                    .remove::<WireframeColor>();
            }
        }
    }

    if keys.just_pressed(KeyCode::F2) {
        config.double_sided = !config.double_sided;
        info!("double_sided: {} → 触发重建", config.double_sided);
        observer.force_rebuild = observer.force_rebuild.wrapping_add(1);
    }
}

/// 当 `TerrainDebugConfig.wireframe` 为 true 时，自动给 terrain mesh 添加 Wireframe 组件。
/// （用于初始启动时的应用，后续切换由 debug_keyboard_toggle 处理）
pub fn apply_debug_wireframe(
    debug_config: Res<TerrainDebugConfig>,
    chunks: Query<&Children, With<TerrainChunk>>,
    global_meshes: Query<Entity, With<GlobalTerrainMesh>>,
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
    for entity in global_meshes.iter() {
        commands.entity(entity).insert((
            Wireframe,
            WireframeColor {
                color: Color::srgba(0.3, 0.9, 0.4, 1.0),
            },
        ));
    }
}
