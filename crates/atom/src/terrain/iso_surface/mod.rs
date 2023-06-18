/// todo: refactor to node graph....
///
///
/// voxel cube mesh: get value and inner or outer from coord(center)
/// marching cube: get value and inner or outer from coord(vertex)
///
/// iso surface density function
/// ios surface mesh generate function
/// chunk store generated mesh.
pub mod noise_surface;
pub mod surface;

use std::{
    fmt::Debug,
    sync::{Arc, RwLock},
};

use bevy::{
    prelude::{App, Plugin},
    utils::HashMap,
};

use self::{noise_surface::NoiseSurface, surface::TerrainSurfaceData};

use super::data::coords::{TerrainChunkCoord, VoxelGradedCoord};

pub struct IsoSurfacePlugin;

impl Plugin for IsoSurfacePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainSurfaceData {
            iso_surface: Arc::new(RwLock::new(NoiseSurface {
                cached_noise_values: HashMap::new(),
                seed: rand::random(),
                y_range: 0..4,
                frequency: 0.05,
                lacunarity: 2.0,
                gain: 10.0,
                octaves: 1,
            })),
        });
    }
}

pub trait IsoSurface: Send + Sync + Debug {
    /// eval and cached or/and add config param to lerp or not?
    /// and param is chunk coord or world location or global coord or other?
    fn eval(&self, voxel_graded_coord: &VoxelGradedCoord) -> f32;

    fn generate(&mut self, chunk_coord: &TerrainChunkCoord);
}
