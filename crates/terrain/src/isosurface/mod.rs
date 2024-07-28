/// 层级划分：
/// 划分为一个个TerrainChunk
/// mesh, mesh cache, octree，作为TerrainChunk的子实体存在。
use std::sync::{Arc, RwLock};

use bevy::prelude::*;
use dc::DualContouringPlugin;
use ecology::EcologyPlugin;
use materials::TerrainMaterialPlugin;
use surface::{
    csg::{csg_noise::WorldGenerator, csg_shapes::CSGPanel},
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
            shape_surface: Arc::new(RwLock::new(ShapeSurface::new(
                Box::new(WorldGenerator::new(0.0)),
                // Box::new(CSGPanel {
                //     location: Vec3::ZERO,
                //     normal: Vec3::Y,
                //     height: 0.0,
                // }),
            ))),
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
