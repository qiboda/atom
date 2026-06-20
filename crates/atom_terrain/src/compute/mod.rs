pub mod gpu;
pub mod sync;
pub mod types;

use bevy::{
    prelude::*,
    render::{Render, RenderApp, RenderStartup},
};

use gpu::{
    init_compute_pipeline, terrain_compute_system, TerrainChunkComputeProgress,
    TerrainChunkMeshBuffers,
};
use sync::TerrainChunksToProcess;

/// 地形 chunk 网格的 GPU compute 管线插件。
/// 在 RenderApp 中注册 GPU compute pipeline、buffer 资源和每帧 compute 系统。
pub struct TerrainChunkMeshComputePlugin;

impl Plugin for TerrainChunkMeshComputePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainChunksToProcess>();

        let render_app = app.sub_app_mut(RenderApp);

        render_app.add_systems(RenderStartup, init_compute_pipeline);
        render_app.add_systems(Render, terrain_compute_system);

        render_app
            .init_resource::<TerrainChunkMeshBuffers>()
            .init_resource::<TerrainChunkComputeProgress>();
    }
}
