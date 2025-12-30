pub mod compute;
pub mod tables;

use bevy::{
    prelude::*,
    render::{RenderApp, extract_component::ExtractComponent},
};
use crossbeam::channel::{Receiver, Sender};

use crate::{
    chunks::loader::{TerrainChunkLoadMsg, TerrainLoadedChunks},
    terrain::TerrainSystems,
};

pub struct TerrainChunkMainMeshData {
    pub mesh: Mesh,
}

pub struct TerrainChunkMeshData {
    pub main_mesh_data: Option<TerrainChunkMainMeshData>,
    pub entity: Entity,
}

/// This will receive asynchronously any data sent from the render world
#[derive(Resource, Deref)]
pub struct TerrainChunkMeshDataReceiver(Receiver<TerrainChunkMeshData>);

/// This will send asynchronously any data to the main world
#[derive(Resource, Deref)]
pub struct TerrainChunkMeshDataSender(Sender<TerrainChunkMeshData>);

#[derive(Component, Default, Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainChunkMeshingState {
    #[default]
    Idle,
    Meshing,
    Seaming,
    Finish,
}

/**
 * 之后改为 RelationShip
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component, ExtractComponent)]
pub struct TerrainChunkSeamMeshes {
    pub right_seam_mesh: Option<Entity>,
    pub top_seam_mesh: Option<Entity>,
    pub front_seam_mesh: Option<Entity>,
}

/**
 * 之后改为 RelationShip
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub struct TerrainChunkMainMesh(Option<Entity>);

fn receive_chunk_load_requests(
    mut commands: Commands,
    mut load_events: MessageReader<TerrainChunkLoadMsg>,
    chunks: Res<TerrainLoadedChunks>,
) {
    for msg in load_events.read() {
        if let Some(entity) = chunks.get(msg.lod, &msg.coord) {
            commands.entity(entity).insert((
                TerrainChunkMeshingState::Idle,
                TerrainChunkMainMesh(None),
                TerrainChunkSeamMeshes {
                    right_seam_mesh: None,
                    top_seam_mesh: None,
                    front_seam_mesh: None,
                },
            ));
        }
    }
}

/**
 * 注册地形 Chunk 网格化相关的系统
 */
pub fn terrain_chunk_meshing_systems(app: &mut App) {
    app.add_systems(
        Update,
        receive_chunk_load_requests.in_set(TerrainSystems::GenerateChunk),
    );

    let (s, r) = crossbeam::channel::unbounded();
    app.insert_resource(TerrainChunkMeshDataReceiver(r));

    let render_app = app.sub_app_mut(RenderApp);
    render_app.insert_resource(TerrainChunkMeshDataSender(s));

    compute::terrain_chunk_meshing_compute_systems(app);
}
