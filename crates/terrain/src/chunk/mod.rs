pub mod chunk_data;
pub mod chunk_mapper;

use bevy::prelude::*;

use chunk_data::TerrainChunkData;
use terrain_core::chunk::coords::TerrainChunkCoord;

#[derive(Bundle, Default)]
pub struct TerrainChunkBundle {
    pub terrain_chunk: TerrainChunk,
    pub terrain_chunk_data: TerrainChunkData,
    pub terrain_chunk_coord: TerrainChunkCoord,
    pub transform_bundle: TransformBundle,
    pub visibility_bundle: VisibilityBundle,
}

#[derive(Debug, Component, Reflect, Default)]
pub struct TerrainChunk;
