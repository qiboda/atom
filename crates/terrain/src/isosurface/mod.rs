/// 层级划分：
/// 划分为一个个TerrainChunk
/// mesh cache, data，octree，都存储在这个entity中。
/// 生成的mesh作为子实体存在。
use std::sync::{Arc, RwLock};

use bevy::prelude::*;
use dc::{dual_contouring, DualContouringPlugin};
use surface::{
    density_function::{Cube, Panel, Sphere},
    shape_surface::ShapeSurface,
};
use surface_nets::SurfaceNetsPlugin;

use super::ecology::EcologyPlugin;

use self::surface::{density_function::NoiseSurface, shape_surface::IsosurfaceContext};

pub mod dc;
pub mod lod;
pub mod mesh;
pub mod state;
pub mod surface;
pub mod surface_nets;

#[derive(Default, Component, Debug, Reflect)]
pub struct IsosurfaceExtract;

#[derive(Default, Debug)]
pub struct IsosurfaceExtractionPlugin;

impl Plugin for IsosurfaceExtractionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(IsosurfaceContext {
            shape_surface: Arc::new(RwLock::new(ShapeSurface {
                density_function: Box::new(Cube),
                // density_function: Box::new(NoiseSurface {
                //     frequency: 0.3,
                //     lacunarity: 0.02,
                //     gain: 5.0,
                //     octaves: 3,
                // }),
                iso_level: Vec3::ZERO,
            })),
        })
        // .add_plugins(SurfaceNetsPlugin)
        .add_plugins(DualContouringPlugin)
        .add_plugins(EcologyPlugin);
    }
}
