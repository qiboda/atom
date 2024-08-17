use bevy::prelude::*;
use dc::gpu_dc::mesh_compute::TerrainChunkMeshComputePlugin;
use pqef::QuadricPlugin;

pub mod dc;
pub mod surface;
pub mod voxel;
pub mod csg;

#[derive(Default, Debug)]
pub struct IsosurfaceExtractionPlugin;

impl Plugin for IsosurfaceExtractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuadricPlugin)
            .add_plugins(TerrainChunkMeshComputePlugin);
    }
}

#[derive(Debug, Reflect, SystemSet, PartialEq, Eq, Hash, Clone)]
pub enum IsosurfaceSystemSet {
    GenerateMainMesh,
    GenerateSeamMesh,
}
