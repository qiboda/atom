use bevy::prelude::*;

use super::{
    chunk_lod::{TerrainChunkAabb, TerrainChunkLod},
    state::{SeamMeshIdGenerator, TerrainChunkAddress, TerrainChunkState},
};

#[derive(Bundle)]
pub struct TerrainChunkBundle {
    pub name: Name,
    pub terrain_chunk: TerrainChunk,
    pub terrain_chunk_lod: TerrainChunkLod,
    pub terrain_chunk_state: TerrainChunkState,
    pub terrain_chunk_aabb: TerrainChunkAabb,
    pub terrain_chunk_address: TerrainChunkAddress,
    pub seam_mesh_id_generator: SeamMeshIdGenerator,
    pub transform_bundle: TransformBundle,
    pub visibility_bundle: VisibilityBundle,
}

impl TerrainChunkBundle {
    pub fn new(terrain_chunk_state: TerrainChunkState) -> Self {
        Self {
            terrain_chunk: TerrainChunk,
            terrain_chunk_lod: default(),
            terrain_chunk_state,
            terrain_chunk_address: TerrainChunkAddress::default(),
            terrain_chunk_aabb: TerrainChunkAabb::default(),
            transform_bundle: default(),
            visibility_bundle: default(),
            seam_mesh_id_generator: SeamMeshIdGenerator::new(),
            name: "terrain chunk".into(),
        }
    }
}

#[derive(Debug, Component, Reflect, Default)]
pub struct TerrainChunk;
