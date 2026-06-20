//! Chunk mesh 数据跨线程收发。
//!
//! 定义从渲染世界回传到主世界的 mesh 数据结构与 channel 收发器。

use bevy::prelude::*;
use crossbeam::channel::{Receiver, Sender};

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

/// 接收主世界的 chunk 加载消息，设置 Meshing 状态
pub fn handle_load_requests(
    mut commands: Commands,
    mut reader: MessageReader<crate::chunk::ChunkLoadMsg>,
    chunks: Res<crate::chunk::TerrainLoadedChunks>,
    setting: Res<crate::setting::TerrainSetting>,
    mut to_process: ResMut<crate::compute::sync::TerrainChunksToProcess>,
) {
    let chunk_size = setting.chunk_size();
    for msg in reader.read() {
        if let Some(entity) = chunks.get(&msg.coord) {
            let world_pos = msg.coord.to_world(chunk_size);
            commands.entity(entity).insert((
                TerrainChunkMeshingState::Meshing,
                Transform::from_translation(world_pos),
            ));
            to_process.pending.insert(entity, world_pos);
        }
    }
}

/// 接收渲染世界发来的 mesh，spawn 新实体并在正确位置渲染
pub fn handle_mesh_data(
    mut commands: Commands,
    receiver: Res<TerrainChunkMeshReceiver>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    while let Ok(data) = receiver.try_recv() {
        let mesh = meshes.add(data.mesh);
        let mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.4, 0.6, 0.3),
            perceptual_roughness: 0.9,
            ..default()
        });
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::from_translation(data.translation),
            Visibility::default(),
        ));
    }
}
