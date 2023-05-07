use bevy::prelude::Bundle;

use super::{
    data::{
        chunk::TerrainChunkData,
        coords::{TerrainChunkCoord, TerrainGlobalCoord, TerrainVoxelCoord},
        terrain::TerrainData,
        visible::TerrainVisibility,
        voxel::TerrainVoxelData,
    },
    noise_config::TerrainNoiseData,
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
    pub terrain_noise_data: TerrainNoiseData,
    pub visible: TerrainVisibility,
}

#[derive(Bundle, Default)]
pub struct TerrainVoxelBundle {
    pub terrain_voxel_data: TerrainVoxelData,
    pub local_coord: TerrainVoxelCoord,
    pub global_coord: TerrainGlobalCoord,
    pub visible: TerrainVisibility,
}
