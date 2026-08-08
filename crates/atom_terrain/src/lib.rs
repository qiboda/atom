#![deny(missing_docs)]
//! Atom 地形系统 — GPU Dual Contouring 程序化地形。
//!
//! 四 pass compute pipeline 在 GPU 上生成 chunk mesh，
//! crossbeam channel 回传 CPU 端用于碰撞检测。
//! 密度场 = y - height_at(x,z)，正值 = air，负值 = solid。
/// 屏幕空间坐标轴指示器
pub mod axis_gizmo;
/// 地表类型系统（与地形高度独立）
pub mod biome;
/// Chunk 加载/卸载管理
pub mod chunk;
/// GPU Mesh compute 管线
pub mod compute;
/// 调试开关（wireframe 等）
pub mod debug;
/// 鼠标点击 SDF 调试
pub mod debug_click;
/// 调试：高度图和地表图导出
pub mod debug_map;
/// 游戏框架系统（俯视角玩家、摄像机、ECS 组件体系）
pub mod game;
/// 观察者驱动的动态加载系统
pub mod loader;
/// Mesh 数据跨线程收发
pub mod mesh;
/// 地形噪声生成
pub mod noise;
/// GPU indirect draw render pipeline
pub mod render;
pub mod screenshot;
/// 地形全局配置
pub mod setting;
use bevy::pbr::wireframe::WireframePlugin;
use bevy::prelude::*;

use axis_gizmo::AxisGizmoPlugin;
use chunk::{ChunkLoadMsg, ChunkUnloadMsg, TerrainLoadedChunks};
use compute::global_compute::TerrainObserver;
use compute::sync::{TerrainChunkProcessReceiver, TerrainChunkProcessSender};
use compute::{GlobalTerrainMeshPlugin, TerrainChunkMeshComputePlugin};
use debug::{TerrainDebugConfig, debug_keyboard_toggle};
use loader::update_grid_chunks;
use mesh::{
    GlobalTerrainMaterial, TerrainChunkMeshReceiver, TerrainChunkMeshSender,
    handle_global_mesh_data, handle_load_requests, handle_mesh_data, handle_unload_requests,
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
            )
                .chain(),
        );

        // ── 调试：wireframe 渲染 ──
        app.add_plugins(WireframePlugin {
            debug_flags: Default::default(),
        });

        // ── 远程截图触发（BRP world.spawn_entity）──
        app.register_type::<screenshot::TakeScreenshot>();
        app.add_systems(Update, screenshot::screenshot_trigger_system);

        // ── GPU compute 管线 ──
        app.add_plugins(TerrainChunkMeshComputePlugin);
    }
}

use compute::chunk::{ChunkLoadRequest, ChunkManager};
use compute::per_chunk::{
    advance_chunk_states, chunk_management_system, init_per_chunk_compute,
    per_chunk_compute_system, slot_sync_system,
};

/// Per-chunk 32³ 地形插件（开放世界，取代 GlobalTerrainPlugin）
pub struct PerChunkTerrainPlugin;

impl Plugin for PerChunkTerrainPlugin {
    fn build(&self, app: &mut App) {
        use bevy::render::{Render, RenderApp, RenderStartup};

        // ── 主世界 ──
        app.insert_resource(ChunkManager::new(50.0, -50.0, 10.0, 42));
        app.init_resource::<TerrainDebugConfig>();
        app.insert_resource(TerrainObserver::default());
        app.init_resource::<ChunkLoadRequest>();
        app.add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
            TerrainObserver,
        >::default());
        app.add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
            ChunkLoadRequest,
        >::default());

        app.add_systems(
            Update,
            (
                update_observer_from_camera,
                chunk_management_system,
                debug::debug_keyboard_toggle,
                debug::draw_debug_gizmos,
                debug_click::debug_click_system,
            ),
        );

        // ── 渲染世界 ──
        let render_app = app.sub_app_mut(RenderApp);
        render_app.insert_resource(ChunkManager::new(50.0, -50.0, 10.0, 42));
        render_app.init_resource::<ChunkLoadRequest>();
        render_app.add_systems(RenderStartup, init_per_chunk_compute);
        render_app.add_systems(
            Render,
            (
                slot_sync_system,
                per_chunk_compute_system,
                advance_chunk_states,
            )
                .chain(),
        );
        app.add_plugins(render::PerChunkRenderPlugin);
        app.add_plugins(axis_gizmo::AxisGizmoPlugin);
    }
}

impl Default for PerChunkTerrainPlugin {
    fn default() -> Self {
        Self
    }
}

/// 全局 Edge Graph DC 地形插件 (Phase 3: observer-centric)。
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
        app.init_resource::<GlobalTerrainMaterial>();

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
            )
                .chain(),
        );

        app.add_plugins(AxisGizmoPlugin);

        // ── 全局 GPU compute 管线 ──
        app.add_plugins(GlobalTerrainMeshPlugin);
    }
}
/// 从主相机 Transform 更新 TerrainObserver 资源。
fn update_observer_from_camera(
    camera: Query<&Transform, (With<Camera3d>, Without<axis_gizmo::GizmoCamera>)>,
    mut observer: ResMut<TerrainObserver>,
) {
    if let Ok(t) = camera.single() {
        observer.position = t.translation;
    }
}
