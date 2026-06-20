pub mod chunk;
pub mod compute;
pub mod loader;
pub mod mesh;
pub mod noise;
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
