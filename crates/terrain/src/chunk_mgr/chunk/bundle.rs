use bevy::prelude::*;

use super::comp::{
    TerrainChunkAabb, TerrainChunkAddress, TerrainChunkMeshEntities, TerrainChunkNeighborLodNodes,
    TerrainChunkSeamLod, TerrainChunkState,
};

#[derive(Bundle)]
pub struct TerrainChunkBundle {
    pub name: Name,
    pub terrain_chunk: TerrainChunk,
    pub terrain_chunk_state: TerrainChunkState,
    pub terrain_chunk_aabb: TerrainChunkAabb,
    pub terrain_chunk_address: TerrainChunkAddress,
    pub terrain_chunk_seam_lod: TerrainChunkSeamLod,
    pub terrain_chunk_mesh_entities: TerrainChunkMeshEntities,
    pub terrain_chunk_neighbor_lod_nodes: TerrainChunkNeighborLodNodes,
    pub transform_bundle: TransformBundle,
    pub visibility_bundle: VisibilityBundle,
}

impl TerrainChunkBundle {
    pub fn new(terrain_chunk_state: TerrainChunkState) -> Self {
        Self {
            terrain_chunk: TerrainChunk,
            terrain_chunk_state,
            terrain_chunk_address: TerrainChunkAddress::default(),
            terrain_chunk_aabb: TerrainChunkAabb::default(),
            transform_bundle: default(),
            visibility_bundle: default(),
            terrain_chunk_seam_lod: TerrainChunkSeamLod([[0; 8]; 8]),
            terrain_chunk_mesh_entities: TerrainChunkMeshEntities::default(),
            terrain_chunk_neighbor_lod_nodes: TerrainChunkNeighborLodNodes::default(),
            name: "terrain chunk".into(),
        }
    }
}

#[derive(Debug, Component, Reflect, Default)]
pub struct TerrainChunk;
