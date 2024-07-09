use std::sync::{Arc, RwLock};

use bevy::prelude::*;
use surface::{density_function::Cube, shape_surface::ShapeSurface};

use super::ecology::EcologyPlugin;

use self::{
    dc::DualContourPlugin,
    surface::{density_function::NoiseSurface, shape_surface::IsosurfaceContext},
};

pub mod dc;
pub mod mesh;
pub mod octree;
pub mod surface;

#[derive(Default, PartialEq, Eq, Debug, Hash, Clone)]
pub enum IsosurfaceExtractionState {
    #[default]
    Sample,
    BuildOctree,
    Extract,
    Meshing,
    CreateMesh,
    Done,
}

#[derive(Default, Component, Debug, Reflect)]
pub struct IsosurfaceExtract;

#[derive(Default, Debug)]
pub struct IsosurfaceExtractionPlugin;

impl Plugin for IsosurfaceExtractionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(IsosurfaceContext {
            shape_surface: Arc::new(RwLock::new(ShapeSurface {
                density_function: Box::new(NoiseSurface {
                    frequency: 0.3,
                    lacunarity: 0.02,
                    gain: 5.0,
                    octaves: 3,
                }),
                iso_level: Vec3::ZERO,
            })),
        })
        .add_plugins(DualContourPlugin)
        .add_plugins(EcologyPlugin);
    }
}
