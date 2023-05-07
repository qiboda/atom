use std::ops::Range;

use bevy::prelude::*;

use super::data::{coords::TerrainVoxelCoord, TERRAIN_VOXEL_NUM_IN_CHUNK};

#[derive(Debug, Resource, Default)]
pub struct TerrainNoiseConfig {
    pub seed: i32,
    pub y_range: Range<i32>,
    pub frequency: f32,
    pub lacunarity: f32,
    pub gain: f32,
    pub octaves: u8,
}

#[derive(Debug, Component, Default)]
pub struct TerrainNoiseData {
    pub noise: Vec<f32>,
}

impl TerrainNoiseData {
    pub fn get_noise_value(&self, voxel_local_coord: &TerrainVoxelCoord) -> Option<&f32> {
        self.noise
            .get(voxel_local_coord.x * TERRAIN_VOXEL_NUM_IN_CHUNK + voxel_local_coord.z)
    }
}
