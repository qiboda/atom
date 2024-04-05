use bevy::prelude::*;

use self::chunk_data::TerrainChunkData;
use terrain_core::chunk::coords::TerrainChunkCoord;

pub mod chunk_data;

#[derive(Bundle, Default)]
pub struct TerrainChunkBundle {
    pub terrain_chunk_data: TerrainChunkData,
    pub terrain_chunk_coord: TerrainChunkCoord,
    pub transform_bundle: TransformBundle,
    pub visibility_bundle: VisibilityBundle,
}

#[derive(Debug, Component, Reflect)]
pub struct TerrainChunk;
