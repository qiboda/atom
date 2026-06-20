//! Chunk mesh 数据跨线程收发。
//!
//! 定义从渲染世界回传到主世界的 mesh 数据结构与 channel 收发器。

use bevy::prelude::*;
use crossbeam::channel::{Receiver, Sender};

use crate::{
    chunk::{
        ChunkLoadMsg, ChunkUnloadMsg, TerrainChunk, TerrainChunkCoord,
        TerrainLoadedChunks,
    },
    compute::sync::{ChunkProcessRequest, TerrainChunkProcessSender},
    setting::TerrainSetting,
};

/// 从 GPU compute 管线回传到主世界的 chunk mesh 数据
pub struct TerrainChunkMeshData {
    /// 生成的网格数据
    pub mesh: Mesh,
    /// chunk 世界坐标偏移
    pub translation: Vec3,
}

/// 主世界端的 mesh 接收器，包装 crossbeam channel `Receiver`
#[derive(Resource, Deref)]
pub struct TerrainChunkMeshReceiver(pub Receiver<TerrainChunkMeshData>);

/// 渲染世界端的 mesh 发送器，包装 crossbeam channel `Sender`
#[derive(Resource, Deref)]
pub struct TerrainChunkMeshSender(pub Sender<TerrainChunkMeshData>);

/// Chunk 网格化状态机
#[derive(Component, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TerrainChunkMeshingState {
    /// 空闲，等待加载
    #[default]
    Idle,
    /// GPU compute 处理中
    Meshing,
    /// compute 完成，等待读回
    Done,
}

/// 接收主世界的 chunk 加载消息，首次加载时创建实体，注册并通过 channel 发送到渲染世界。
pub fn handle_load_requests(
    mut commands: Commands,
    mut reader: MessageReader<ChunkLoadMsg>,
    setting: Res<TerrainSetting>,
    mut loaded_chunks: ResMut<TerrainLoadedChunks>,
    sender: Res<TerrainChunkProcessSender>,
) {
    let chunk_size = setting.chunk_size();
    for msg in reader.read() {
        if loaded_chunks.contains(&msg.coord) {
            // 已在队列中，跳过（可能是前一帧刚创建的）
            continue;
        }
        let world_pos = msg.coord.to_world(chunk_size);
        let entity = commands
            .spawn((
                TerrainChunk,
                TerrainChunkCoord(msg.coord.0),
                TerrainChunkMeshingState::Meshing,
                Transform::IDENTITY, // 顶点已是世界坐标，父 entity 不加偏移
                Visibility::default(),
            ))
            .id();
        loaded_chunks.insert(msg.coord, entity);
        let _ = sender.send(ChunkProcessRequest::Load {
            entity,
            world_min: world_pos,
        });
    }
}

/// 接收卸载消息: despawn chunk entity（含 children mesh），清理注册表，通知渲染世界释放 GPU buffer。
pub fn handle_unload_requests(
    mut commands: Commands,
    mut reader: MessageReader<ChunkUnloadMsg>,
    mut loaded_chunks: ResMut<TerrainLoadedChunks>,
    sender: Res<TerrainChunkProcessSender>,
) {
    for msg in reader.read() {
        if let Some(entity) = loaded_chunks.remove(&msg.coord) {
            commands.entity(entity).despawn();
            let _ = sender.send(ChunkProcessRequest::Unload { entity });
        }
    }
}

/// 接收渲染世界发来的 mesh，spawn 为 chunk entity 的子实体
pub fn handle_mesh_data(
    mut commands: Commands,
    receiver: Res<TerrainChunkMeshReceiver>,
    setting: Res<TerrainSetting>,
    loaded_chunks: Res<TerrainLoadedChunks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    while let Ok(data) = receiver.try_recv() {
        let mesh = meshes.add(data.mesh);
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.75, 0.8),
            perceptual_roughness: 0.3,
            ..default()
        });
        // 通过 translation 反查 TerrainChunkCoord，找到父 chunk entity
        let coord = TerrainChunkCoord::from_world(data.translation, setting.chunk_size());
        if let Some(chunk_entity) = loaded_chunks.get(&coord) {
            commands.entity(chunk_entity).with_children(|parent| {
                parent.spawn((
                    Mesh3d(mesh),
                    MeshMaterial3d(mat),
                    Transform::IDENTITY, // 顶点已由 shader 转为世界坐标
                    Visibility::default(),
                ));
            });
            commands.entity(chunk_entity).insert(TerrainChunkMeshingState::Done);
        }
    }
}

/// 全局 mesh 标记组件（Phase 3 global pipeline）。
#[derive(Component)]
pub struct GlobalTerrainMesh;

/// 接收渲染世界发来的全局 mesh（Phase 3），直接 spawn 实体。
///
/// 与 `handle_mesh_data` 不同，不依赖 TerrainLoadedChunks，
/// mesh 直接独立 spawn（顶点已为世界坐标）。
pub fn handle_global_mesh_data(
    mut commands: Commands,
    receiver: Res<TerrainChunkMeshReceiver>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mesh_entity: Local<Option<Entity>>,
) {
    while let Ok(data) = receiver.try_recv() {
        let mesh = meshes.add(data.mesh);
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.7, 0.75, 0.8),
            perceptual_roughness: 0.3,
            ..default()
        });

        // 如果已有旧 mesh 实体，despawn 它
        if let Some(old) = mesh_entity.take() {
            commands.entity(old).despawn();
        }

        let entity = commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::IDENTITY,
            Visibility::default(),
            GlobalTerrainMesh,
        )).id();
        *mesh_entity = Some(entity);
    }
}
