use bevy::prelude::*;

use terrain_core::chunk::coords::TerrainChunkCoord;

use super::{chunk_lod::TerrainChunkLod, state::TerrainChunkState};

#[derive(Bundle)]
pub struct TerrainChunkBundle {
    pub terrain_chunk: TerrainChunk,
    pub terrain_chunk_lod: TerrainChunkLod,
    pub terrain_chunk_state: TerrainChunkState,
    pub chunk_coord: TerrainChunkCoord,
    pub transform_bundle: TransformBundle,
    pub visibility_bundle: VisibilityBundle,
}

impl TerrainChunkBundle {
    pub fn new(terrain_chunk_state: TerrainChunkState) -> Self {
        Self {
            terrain_chunk: TerrainChunk,
            terrain_chunk_lod: default(),
            terrain_chunk_state,
            chunk_coord: default(),
            transform_bundle: default(),
            visibility_bundle: default(),
        }
    }
}

#[derive(Debug, Component, Reflect, Default)]
pub struct TerrainChunk;
