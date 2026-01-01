use bevy::{diagnostic::FrameCount, prelude::*};

use crate::chunks::{
    chunk::{TerrainChunk, TerrainChunkCoord},
    loader::{TerrainChunkLoadMsg, TerrainLoadedChunks},
    mesh::{
        channel::TerrainChunkMeshDataReceiver,
        components::TerrainChunkMeshingState,
        visual::{TerrainChunkLogic, TerrainChunkMesh, TerrainChunkVisual},
    },
};

/// 接收 chunk 加载请求，初始化网格状态
/// 通常不需要处理卸载请求，会自动卸载所有关联的实体
pub fn receive_chunk_load_requests(
    mut commands: Commands,
    mut load_events: MessageReader<TerrainChunkLoadMsg>,
    chunks: Res<TerrainLoadedChunks>,
    frame_count: Res<FrameCount>,
) {
    for msg in load_events.read() {
        if let Some(entity) = chunks.get(&msg.coord) {
            commands
                .entity(entity)
                .insert((TerrainChunkMeshingState::Meshing,));
            debug!(
                "frame count: {} Chunk {:?} load request received, set meshing state to Meshing",
                frame_count.0, msg.coord
            );
        }
    }
}

/// 接收从渲染线程发送的网格数据，并创建网格资源
pub fn receive_chunk_mesh_data(
    mut commands: Commands,
    mesh_data_receiver: Res<TerrainChunkMeshDataReceiver>,
    chunk_coords_query: Query<
        (&TerrainChunkCoord, Option<&TerrainChunkVisual>),
        With<TerrainChunk>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    frame_count: Res<FrameCount>,
) {
    while let Ok(mesh_data) = mesh_data_receiver.try_recv() {
        if let Ok((chunk_coord, visual)) = chunk_coords_query.get(mesh_data.chunk_entity) {
            if visual.is_some() {
                error!(
                    "frame count: {} Entity: {:?} Chunk {:?} already has a visual entity, skipping mesh data application",
                    frame_count.0, mesh_data.chunk_entity, *chunk_coord
                );
                continue;
            }

            let mesh_handle = meshes.add(mesh_data.mesh);

            let material_handle = materials.add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 1.0,
                ..Default::default()
            });

            commands
                .entity(mesh_data.chunk_entity)
                .insert(TerrainChunkMeshingState::Idle)
                .with_related::<TerrainChunkLogic>((
                    Name::new(format!("Terrain Chunk Mesh_{:?}", *chunk_coord)),
                    Mesh3d(mesh_handle),
                    MeshMaterial3d(material_handle),
                    TerrainChunkMesh,
                ));
            info!(
                "frame count: {} Chunk {:?} mesh data received and visual entity created.",
                frame_count.0, *chunk_coord
            );
        } else {
            error!(
                "frame count: {} Received mesh data for unknown chunk entity: {:?}, maybe it was unloaded before mesh creation.",
                frame_count.0, mesh_data.chunk_entity
            );
        }
    }
}
