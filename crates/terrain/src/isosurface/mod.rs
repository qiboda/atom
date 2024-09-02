use bevy::prelude::*;
use dc::gpu_dc::mesh_compute::TerrainChunkMeshComputePlugin;
use pqef::QuadricPlugin;
use strum::EnumCount;

use crate::{map::topography::MapFlatTerrainType, tables::SubNodeIndex};

pub mod csg;
pub mod dc;
pub mod surface;

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IsosurfaceSide {
    Inside,
    Outside,
}

impl From<bool> for IsosurfaceSide {
    fn from(value: bool) -> Self {
        if value {
            IsosurfaceSide::Outside
        } else {
            IsosurfaceSide::Inside
        }
    }
}

pub fn select_voxel_biome(biomes: [MapFlatTerrainType; SubNodeIndex::COUNT]) -> MapFlatTerrainType {
    let mut biome_count = [0; MapFlatTerrainType::MAX];
    if biomes.iter().any(|x| x.is_surface_type()) {
        for biome in biomes.iter() {
            if biome.is_surface_type() {
                biome_count[*biome as usize] += 1;
            }
        }
    } else {
        for biome in biomes.iter() {
            biome_count[*biome as usize] += 1;
        }
    }

    let mut terrain_type = MapFlatTerrainType::Ocean;
    let mut max_count = 0;
    for (terrain, i) in biome_count.iter().enumerate() {
        if *i > max_count {
            max_count = *i;
            terrain_type = MapFlatTerrainType::from_repr(terrain).unwrap()
        }
    }

    terrain_type
}
