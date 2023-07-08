use std::ops::Range;

use bevy::{prelude::info, utils::HashMap};
use simdnoise::NoiseBuilder;

use crate::terrain::data::{
    coords::{TerrainChunkCoord, VoxelGradedCoord},
    TERRAIN_VOXEL_NUM_IN_CHUNK,
};

use super::IsoSurface;

#[derive(Debug, Default)]
pub struct NoiseSurface {
    pub cached_noise_values: HashMap<TerrainChunkCoord, Vec<f32>>,

    pub seed: i32,
    pub y_range: Range<i32>,
    pub frequency: f32,
    pub lacunarity: f32,
    pub gain: f32,
    pub octaves: u8,
}

impl NoiseSurface {
    fn build_noise(&self, chunk_coord: &TerrainChunkCoord) -> (Vec<f32>, f32, f32) {
        let offset_x = (chunk_coord.x * TERRAIN_VOXEL_NUM_IN_CHUNK as i64) as f32;
        let offset_z = (chunk_coord.z * TERRAIN_VOXEL_NUM_IN_CHUNK as i64) as f32;

        NoiseBuilder::fbm_2d_offset(
            offset_x,
            TERRAIN_VOXEL_NUM_IN_CHUNK,
            offset_z,
            TERRAIN_VOXEL_NUM_IN_CHUNK,
        )
        .with_seed(self.seed)
        .with_freq(self.frequency)
        .with_gain(self.gain)
        .with_lacunarity(self.lacunarity)
        .with_octaves(self.octaves)
        .generate()
    }
}

impl IsoSurface for NoiseSurface {
    fn eval(&self, voxel_graded_coord: &VoxelGradedCoord) -> f32 {
        self.cached_noise_values
            .get(&voxel_graded_coord.chunk_coord)
            .map(|chunk_noise| {
                // warn!("voxel index: {:?}", voxel_graded_coord.voxel_coord);
                // warn!("voxel chunk index: {:?}", voxel_graded_coord.chunk_coord);
                let voxel_index = voxel_graded_coord.voxel_coord.x
                    + voxel_graded_coord.voxel_coord.z * TERRAIN_VOXEL_NUM_IN_CHUNK;
                chunk_noise[voxel_index]
            })
            .unwrap_or_else(|| {
                info!(
                    "No noise value found for chunk coord {:?}",
                    voxel_graded_coord.chunk_coord
                );
                0.0
            })
    }

    fn generate(&mut self, chunk_coord: &TerrainChunkCoord) {
        let (noise, _min, _max) = self.build_noise(&chunk_coord);

        let chunk_noise =
            self.cached_noise_values
                .entry_ref(chunk_coord)
                .or_insert(Vec::with_capacity(
                    TERRAIN_VOXEL_NUM_IN_CHUNK * TERRAIN_VOXEL_NUM_IN_CHUNK,
                ));

        if chunk_noise.len() != 0 {
            return;
        }

        // warn!("Generating noise for chunk coord {:?}", chunk_coord);

        let scaled_min = self.y_range.start as f32;
        let scaled_max = self.y_range.end as f32;
        for value in noise.iter() {
            chunk_noise.push(scaled_min + (scaled_max - scaled_min) * *value);
        }
    }
}
