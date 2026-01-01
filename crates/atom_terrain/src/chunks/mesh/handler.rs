use bevy::prelude::*;

use crate::chunks::{
    loader::{TerrainChunkLoadMsg, TerrainLoadedChunks},
    mesh::{
        channel::TerrainChunkMeshDataReceiver,
        components::TerrainChunkMeshingState,
        visual::{TerrainChunkMeshHandle, TerrainChunkVisual},
    },
};

/// 接收 chunk 加载请求，初始化网格状态
/// 通常不需要处理卸载请求，会自动卸载所有关联的实体
pub fn receive_chunk_load_requests(
    mut commands: Commands,
    mut load_events: MessageReader<TerrainChunkLoadMsg>,
    chunks: Res<TerrainLoadedChunks>,
) {
    for msg in load_events.read() {
        if let Some(entity) = chunks.get(&msg.coord) {
            commands
                .entity(entity)
                .insert((TerrainChunkMeshingState::Meshing,));
        }
    }
}

/// 接收从渲染线程发送的网格数据，并创建网格资源
pub fn receive_chunk_mesh_data(
    mut commands: Commands,
    mesh_data_receiver: Res<TerrainChunkMeshDataReceiver>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    while let Ok(mesh_data) = mesh_data_receiver.try_recv() {
        let mesh_handle = meshes.add(mesh_data.mesh);
        commands
            .entity(mesh_data.chunk_entity)
            .insert(TerrainChunkMeshingState::Finish)
            .with_related::<TerrainChunkVisual>(TerrainChunkMeshHandle(mesh_handle));
    }
}
