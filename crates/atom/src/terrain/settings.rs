use bevy::prelude::*;

#[derive(Resource, Debug)]
pub struct TerrainSettings {
    chunk_voxel_size: f32,
    chunk_voxel_num: u32,
}

impl TerrainSettings {
    pub fn new(chunk_voxel_size: f32, chunk_voxel_num: u32) -> Self {
        debug_assert!(chunk_voxel_num.is_power_of_two());
        Self {
            chunk_voxel_size,
            chunk_voxel_num,
        }
    }
}

impl TerrainSettings {
    #[inline]
    pub fn get_chunk_voxel_size(&self) -> f32 {
        self.chunk_voxel_size
    }

    #[inline]
    pub fn get_chunk_voxel_num(&self) -> u32 {
        self.chunk_voxel_num
    }

    pub fn get_chunk_size(&self) -> f32 {
        self.chunk_voxel_size * self.chunk_voxel_num as f32
    }
}
