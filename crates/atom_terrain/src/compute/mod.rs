pub mod global_compute;
pub mod global_pool;
pub mod gpu;
pub mod sync;
pub mod types;

use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResourcePlugin,
        renderer::RenderDevice,
        Render, RenderApp, RenderStartup,
    },
};

use crate::setting::TerrainSetting;
use global_compute::{
    init_global_compute_pipeline, global_compute_system,
    TerrainObserver, GlobalComputeState, GlobalStagingState,
};
use global_pool::GlobalMeshPool;
use gpu::{
    init_compute_pipeline, terrain_compute_system, TerrainChunkComputeProgress,
    TerrainChunkMeshBuffers, TerrainChunkStagingBuffers,
};

/// 地形 chunk 网格的 GPU compute 管线插件 (Phase 2: per-chunk)。
pub struct TerrainChunkMeshComputePlugin;

impl Plugin for TerrainChunkMeshComputePlugin {
    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);

        render_app.insert_resource(TerrainSetting::default());
        render_app.init_resource::<TerrainChunkMeshBuffers>();
        render_app.init_resource::<TerrainChunkComputeProgress>();
        render_app.init_resource::<TerrainChunkStagingBuffers>();

        render_app.add_systems(RenderStartup, init_compute_pipeline);
        render_app.add_systems(Render, terrain_compute_system);
    }
}

/// 全局 Edge Graph DC 管线插件 (Phase 3: observer-centric, 无 chunk 边界)。
///
/// 替换 `TerrainChunkMeshComputePlugin`，使用全局 edge graph + atomic counter +
/// 单次 mesh 渲染替代 per-chunk 管线。
pub struct GlobalTerrainMeshPlugin;

impl Plugin for GlobalTerrainMeshPlugin {
    fn build(&self, app: &mut App) {
        // ── 主世界 ──
        app.insert_resource(TerrainObserver::default());
        app.add_plugins(ExtractResourcePlugin::<TerrainObserver>::default());

        // ── 渲染世界 ──
        let render_app = app.sub_app_mut(RenderApp);
        render_app.insert_resource(TerrainSetting::default());
        render_app.init_resource::<GlobalComputeState>();
        render_app.init_resource::<GlobalStagingState>();

        // RenderStartup: 创建 GlobalMeshPool → 初始化 compute pipeline
        render_app.add_systems(
            RenderStartup,
            (init_global_pool, init_global_compute_pipeline).chain(),
        );
        render_app.add_systems(Render, global_compute_system);
    }
}

/// 创建 GlobalMeshPool（依赖 RenderDevice + TerrainSetting）。
fn init_global_pool(mut commands: Commands, device: Res<RenderDevice>, setting: Res<TerrainSetting>) {
    let grid_size = 50u32; // inner 48 + 1-voxel shell 两侧 = 50 (25m 半径)
    let pool = GlobalMeshPool::new(&device, setting.voxel_size, grid_size);
    info!(
        "GlobalMeshPool: grid_size={grid_size}, vertex_cap={}, index_cap={}",
        pool.vertex_capacity, pool.index_capacity
    );
    commands.insert_resource(pool);
}
