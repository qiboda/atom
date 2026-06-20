use bevy::prelude::*;
use crossbeam::channel::{Receiver, Sender};

pub struct TerrainChunkMeshData {
    pub mesh: Mesh,
    pub chunk_entity: Entity,
}

#[derive(Resource, Deref)]
pub struct TerrainChunkMeshReceiver(pub Receiver<TerrainChunkMeshData>);

#[derive(Resource, Deref)]
pub struct TerrainChunkMeshSender(pub Sender<TerrainChunkMeshData>);

#[derive(Component, Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum TerrainChunkMeshingState {
    #[default]
    Idle,
    Meshing,
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

/// 接收渲染世界发来的 mesh，附加到 chunk 实体上
pub fn handle_mesh_data(
    mut commands: Commands,
    receiver: Res<TerrainChunkMeshReceiver>,
    chunks: Query<&crate::chunk::TerrainChunkCoord>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    while let Ok(data) = receiver.try_recv() {
        if chunks.contains(data.chunk_entity) {
            let mesh = meshes.add(data.mesh);
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.4, 0.6, 0.3),
                perceptual_roughness: 0.9,
                ..default()
            });
            commands.entity(data.chunk_entity).insert((
                TerrainChunkMeshingState::Idle,
                Mesh3d(mesh),
                MeshMaterial3d(mat),
            ));
        }
    }
}
