use bevy::prelude::*;

use self::{chunk::TerrainChunkData, coords::TerrainChunkCoord};

use super::isosurface::IsosurfaceExtractionState;

pub mod chunk;
pub mod coords;

#[derive(Bundle, Default)]
pub struct TerrainChunkBundle {
    pub terrain_chunk_data: TerrainChunkData,
    pub terrain_chunk_coord: TerrainChunkCoord,
    pub state: IsosurfaceExtractionState,
    pub transform_bundle: TransformBundle,
    pub visibility_bundle: VisibilityBundle,
}

#[derive(Debug, Component)]
pub struct TerrainChunk;
