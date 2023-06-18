use std::sync::{Arc, RwLock};

use bevy::prelude::*;

use crate::terrain::data::coords::{TerrainChunkCoord, VoxelGradedCoord};

use super::IsoSurface;

#[derive(Debug, Resource)]
pub struct TerrainSurfaceData {
    pub iso_surface: Arc<RwLock<dyn IsoSurface>>,
}

impl TerrainSurfaceData {
    pub fn get_surface_value(&self, voxel_graded_coord: &VoxelGradedCoord) -> f32 {
        let surface = self.iso_surface.read().unwrap();
        surface.eval(voxel_graded_coord)
    }

    pub fn generate_surface_value(&self, terrain_chunk_coord: &TerrainChunkCoord) {
        let mut surface = self.iso_surface.write().unwrap();
        surface.generate(terrain_chunk_coord);
    }
}
