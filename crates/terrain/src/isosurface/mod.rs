/// 层级划分：
/// 划分为一个个TerrainChunk
/// mesh, mesh cache, octree，作为TerrainChunk的子实体存在。
use std::sync::{Arc, RwLock};

use bevy::prelude::*;
use dc::DualContouringPlugin;
use ecology::EcologyPlugin;
use materials::TerrainMaterialPlugin;
use surface::{
    density_function::{NoiseSurface, Panel},
    shape_surface::ShapeSurface,
};

use self::surface::shape_surface::IsosurfaceContext;

pub mod comp;
pub mod dc;
pub mod ecology;
pub mod materials;
pub mod mesh;
pub mod surface;

#[derive(Default, Debug)]
pub struct IsosurfaceExtractionPlugin;

impl Plugin for IsosurfaceExtractionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(IsosurfaceContext {
            shape_surface: Arc::new(RwLock::new(ShapeSurface {
                density_function: Box::new(Panel),
                // density_function: Box::new(NoiseSurface {
                //     frequency: 0.3,
                //     lacunarity: 0.02,
                //     gain: 5.0,
                //     octaves: 3,
                // }),
                iso_level: Vec3::ZERO,
            })),
        })
        .add_plugins(EcologyPlugin)
        .add_plugins(TerrainMaterialPlugin)
        .add_plugins(DualContouringPlugin);
    }
}

#[derive(Debug, Reflect, SystemSet, PartialEq, Eq, Hash, Clone)]
pub enum IsosurfaceSystemSet {
    GenerateMainMesh,
    GenerateSeamMesh,
}
