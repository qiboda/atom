use bevy::prelude::Bundle;

use super::chunk::{
    chunk::TerrainChunkData, coords::TerrainChunkCoord, terrain::TerrainData,
    visible::TerrainVisibility,
};

#[derive(Bundle, Default)]
pub struct TerrainBundle {
    pub terrain_data: TerrainData,
    pub voxel_visibility: TerrainVisibility,
}

#[derive(Bundle, Default)]
pub struct TerrainChunkBundle {
    pub terrain_chunk_data: TerrainChunkData,
    pub terrain_chunk_coord: TerrainChunkCoord,
    pub visible: TerrainVisibility,
}
