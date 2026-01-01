use bevy::prelude::*;
use crossbeam::channel::{Receiver, Sender};

pub struct TerrainChunkMeshData {
    pub mesh: Mesh,
    // Chunk的实体
    pub chunk_entity: Entity,
}

/// This will receive asynchronously any data sent from the render world
#[derive(Resource, Deref)]
pub struct TerrainChunkMeshDataReceiver(pub Receiver<TerrainChunkMeshData>);

/// This will send asynchronously any data to the main world
#[derive(Resource, Deref)]
pub struct TerrainChunkMeshDataSender(pub Sender<TerrainChunkMeshData>);
