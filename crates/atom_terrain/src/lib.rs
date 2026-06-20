#![deny(missing_docs)]
//! Atom 地形系统 — GPU Dual Contouring 程序化地形。
//!
//! 四 pass compute pipeline 在 GPU 上生成 chunk mesh，
//! crossbeam channel 回传 CPU 端用于碰撞检测。
//! 密度场 = y - height_at(x,z)，正值 = air，负值 = solid。
/// Chunk 加载/卸载管理
pub mod chunk;
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

use chunk::{ChunkLoadMsg, ChunkUnloadMsg, TerrainLoadedChunks};
use compute::TerrainChunkMeshComputePlugin;
use loader::update_grid_chunks;
use mesh::{
    handle_load_requests, handle_mesh_data, TerrainChunkMeshReceiver, TerrainChunkMeshSender,
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
        app.add_message::<ChunkLoadMsg>();
        app.add_message::<ChunkUnloadMsg>();

        // ── 渲染 → 主世界 mesh 数据 channel ──
        let (tx, rx) = crossbeam::channel::unbounded();
        app.insert_resource(TerrainChunkMeshReceiver(rx));
        let render_app = app.sub_app_mut(bevy::render::RenderApp);
        render_app.insert_resource(TerrainChunkMeshSender(tx));

        // ── 主世界系统 ──
        app.add_systems(
            Update,
            (
                update_grid_chunks,
                handle_load_requests,
                handle_mesh_data,
            )
                .chain(),
        );

        // ── GPU compute 管线 ──
        app.add_plugins(TerrainChunkMeshComputePlugin);
    }
}
