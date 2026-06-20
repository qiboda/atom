pub mod gpu;
pub mod sync;
pub mod types;

use bevy::{
    prelude::*,
    render::{Render, RenderApp, RenderStartup},
};

use crate::setting::TerrainSetting;
use gpu::{
    init_compute_pipeline, terrain_compute_system, TerrainChunkComputeProgress,
    TerrainChunkMeshBuffers, TerrainChunkStagingBuffers,
};
use sync::TerrainChunksToProcess;

/// 渲染 world 启动时创建测试用的 chunk entity 和 pending 条目。
/// GPU value noise height_at(0,0) ≈ -26，chunk 高 8，表面在 [-26, -18] 内。
pub fn setup_test_chunk(
    mut commands: Commands,
    mut to_process: ResMut<TerrainChunksToProcess>,
) {
    let entity = commands.spawn_empty().id();
    let pos = Vec3::new(0.0, -26.0, 0.0);
    to_process.pending.insert(entity, pos);
    info!("[ComputePlugin] 注入测试 chunk entity={entity:?} at (0,-26,0)");
}

/// 地形 chunk 网格的 GPU compute 管线插件。
/// 在 RenderApp 中注册 GPU compute pipeline、buffer 资源和每帧 compute 系统。
pub struct TerrainChunkMeshComputePlugin;

impl Plugin for TerrainChunkMeshComputePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainChunksToProcess>();

        let render_app = app.sub_app_mut(RenderApp);

        // 直接插入到 render world（ExtractResourcePlugin 在 0.19 的 sub-app 时序中有问题）
        render_app.insert_resource(TerrainSetting::default());
        render_app.init_resource::<TerrainChunksToProcess>();
        render_app.init_resource::<TerrainChunkMeshBuffers>();
        render_app.init_resource::<TerrainChunkComputeProgress>();
        render_app.init_resource::<TerrainChunkStagingBuffers>();

        render_app.add_systems(RenderStartup, (init_compute_pipeline, setup_test_chunk).chain());
        render_app.add_systems(Render, terrain_compute_system);
    }
}
