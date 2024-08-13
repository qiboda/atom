use bevy::prelude::*;
use dc::gpu_dc::mesh_compute::TerrainChunkMeshComputePlugin;
use ecology::EcologyPlugin;
use materials::TerrainMaterialPlugin;
use pqef::QuadricPlugin;

pub mod dc;
pub mod ecology;
pub mod materials;
pub mod surface;
pub mod voxel;

#[derive(Default, Debug)]
pub struct IsosurfaceExtractionPlugin;

impl Plugin for IsosurfaceExtractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(QuadricPlugin)
            .add_plugins(EcologyPlugin)
            .add_plugins(TerrainMaterialPlugin)
            .add_plugins(TerrainChunkMeshComputePlugin);
    }
}

#[derive(Debug, Reflect, SystemSet, PartialEq, Eq, Hash, Clone)]
pub enum IsosurfaceSystemSet {
    GenerateMainMesh,
    GenerateSeamMesh,
}
