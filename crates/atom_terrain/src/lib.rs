#![deny(missing_docs)]
//! Atom 地形系统 — GPU Dual Contouring 程序化地形。
//!
//! 四 pass compute pipeline 在 GPU 上生成 chunk mesh，
//! crossbeam channel 回传 CPU 端用于碰撞检测。
//! 密度场 = y - height_at(x,z)，正值 = air，负值 = solid。
/// Chunk 加载/卸载管理
pub mod chunk;
/// 调试开关（wireframe 等）
pub mod debug;
/// GPU Mesh compute 管线
pub mod compute;
/// 观察者驱动的动态加载系统
pub mod loader;
/// Mesh 数据跨线程收发
pub mod mesh;
/// 地形噪声生成
pub mod noise;
/// 地形全局配置
pub mod setting;

use bevy::prelude::*;
use bevy::pbr::wireframe::WireframePlugin;

use chunk::{ChunkLoadMsg, ChunkUnloadMsg, TerrainLoadedChunks};
use compute::global_compute::TerrainObserver;
use compute::sync::{TerrainChunkProcessReceiver, TerrainChunkProcessSender};
use compute::{GlobalTerrainMeshPlugin, TerrainChunkMeshComputePlugin};
use debug::{apply_debug_wireframe, debug_keyboard_toggle, TerrainDebugConfig};
use loader::update_grid_chunks;
use mesh::{
    handle_global_mesh_data, handle_load_requests, handle_mesh_data,
    handle_unload_requests, TerrainChunkMeshReceiver, TerrainChunkMeshSender,
};
use setting::TerrainSetting;

/// 地形系统总插件
pub struct TerrainPlugin;

impl Default for TerrainPlugin {
    fn default() -> Self {
        Self
    }
}

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        // ── 主世界资源 ──
        app.insert_resource(TerrainSetting::default());
        app.init_resource::<TerrainLoadedChunks>();
        app.init_resource::<TerrainDebugConfig>();
        app.add_message::<ChunkLoadMsg>();
        app.add_message::<ChunkUnloadMsg>();

        // ── 渲染 → 主世界 mesh 数据 channel ──
        let (mesh_tx, mesh_rx) = crossbeam::channel::unbounded();
        app.insert_resource(TerrainChunkMeshReceiver(mesh_rx));

        // ── 主世界 → 渲染 chunk 处理请求 channel ──
        let (proc_tx, proc_rx) = crossbeam::channel::unbounded();
        app.insert_resource(TerrainChunkProcessSender(proc_tx));

        let render_app = app.sub_app_mut(bevy::render::RenderApp);
        render_app.insert_resource(TerrainChunkMeshSender(mesh_tx));
        render_app.insert_resource(TerrainChunkProcessReceiver(proc_rx));

        // ── 主世界系统 ──
        app.add_systems(
            Update,
            (
                update_grid_chunks,
                handle_load_requests,
                handle_unload_requests,
                handle_mesh_data,
                debug_keyboard_toggle,
                apply_debug_wireframe,
            )
                .chain(),
        );

        // ── 调试：wireframe 渲染 ──
        app.add_plugins(WireframePlugin {
            debug_flags: Default::default(),
        });

        // ── GPU compute 管线 ──
        app.add_plugins(TerrainChunkMeshComputePlugin);
    }
}

/// 全局 Edge Graph DC 地形插件 (Phase 3: observer-centric)。
///
/// 使用全局 edge graph + atomic counter + 单次 mesh 渲染，
/// 替代 per-chunk 管线，消除 chunk 边界 seam。
pub struct GlobalTerrainPlugin;

impl Default for GlobalTerrainPlugin {
    fn default() -> Self {
        Self
    }
}

impl Plugin for GlobalTerrainPlugin {
    fn build(&self, app: &mut App) {
        // ── 主世界资源 ──
        app.insert_resource(TerrainSetting::default());
        app.init_resource::<TerrainDebugConfig>();

        // ── 渲染 → 主世界 mesh 数据 channel ──
        let (mesh_tx, mesh_rx) = crossbeam::channel::unbounded();
        app.insert_resource(TerrainChunkMeshReceiver(mesh_rx));

        let render_app = app.sub_app_mut(bevy::render::RenderApp);
        render_app.insert_resource(TerrainChunkMeshSender(mesh_tx));

        // ── 主世界系统 ──
        app.add_systems(
            Update,
            (
                update_observer_from_camera,
                handle_global_mesh_data,
                debug_keyboard_toggle,
                apply_debug_wireframe,
            )
                .chain(),
        );

        // ── 调试：wireframe 渲染 ──
        app.add_plugins(WireframePlugin {
            debug_flags: Default::default(),
        });

        // ── 全局 GPU compute 管线 ──
        app.add_plugins(GlobalTerrainMeshPlugin);
    }
}

/// 从主相机 Transform 更新 TerrainObserver 资源。
fn update_observer_from_camera(
    camera: Query<&Transform, With<Camera3d>>,
    mut observer: ResMut<TerrainObserver>,
) {
    if let Ok(t) = camera.single() {
        observer.position = t.translation;
    }
}
