//! 地形 MVP 示例 — 创建单个 chunk 并通过 GPU compute 生成 mesh。
use bevy::prelude::*;
use atom_terrain::{
    chunk::{TerrainChunk, TerrainChunkCoord},
    compute::sync::TerrainChunksToProcess,
    loader::{TerrainObserver, TerrainObserverConfig},
    mesh::TerrainChunkMeshingState,
    TerrainPlugin,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(TerrainPlugin::default());
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut to_process: ResMut<TerrainChunksToProcess>,
) {
    // 摄像机 — 对准测试 chunk 中心 (chunk 跨度 [-26,-18], 中心 ≈ -22)
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, -12.0, 15.0).looking_at(Vec3::new(0.0, -22.0, 0.0), Vec3::Y),
        TerrainObserver,
        TerrainObserverConfig::default(),
    ));

    // 方向光
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 创建单个 chunk 并直接加入处理队列
    let entity = commands.spawn((
        TerrainChunk,
        TerrainChunkCoord::new(0, 0, 0),
        TerrainChunkMeshingState::Meshing,
        Transform::from_translation(Vec3::ZERO),
    )).id();

    to_process.pending.insert(entity, Vec3::ZERO);
}
