use bevy::prelude::*;
use meshing::MeshingPlugin;
use sample::SampleSurfacePlugin;
use surface::shape_surface::ShapeSurface;

use self::{cms::ExtractPlugin, octree::OctreePlugin, surface::densy_function::NoiseSurface};

use super::TerrainSystemSet;

pub mod cms;
pub mod gpu;
pub mod meshing;
pub mod octree;
pub mod sample;
pub mod surface;

#[derive(PartialEq, Eq, Debug, Hash, Clone, SystemSet)]
pub enum IsosurfaceExtractionSet {
    Sample,
    BuildOctree,
    Extract,
    Meshing,
}

#[derive(Default, Component, Debug)]
pub struct IsosurfaceExtract;

#[derive(PartialEq, Eq, Debug, Clone, Default)]
pub enum BuildOctreeState {
    #[default]
    Build,
    MarkTransitFace,
}

#[derive(PartialEq, Eq, Debug, Clone, Component, Default)]
pub enum IsosurfaceExtractionState {
    #[default]
    Sample,
    BuildOctree(BuildOctreeState),
    Extract,
    Meshing,
    MeshCreate,
    Done,
}

#[derive(Default, Debug)]
pub struct IsosurfaceExtractionPlugin;

impl Plugin for IsosurfaceExtractionPlugin {
    fn build(&self, app: &mut App) {
        info!("add IsosurfaceExtractionPlugin");
        app.insert_resource(ShapeSurface {
            densy_function: Box::new(NoiseSurface {
                seed: rand::random(),
                frequency: 0.1,
                lacunarity: 0.01,
                gain: 5.0,
                octaves: 3,
            }),
            // densy_function: Box::new(Sphere),
            // densy_function: Box::new(Panel),
            // densy_function: Box::new(Cube),
            iso_level: Vec3::ZERO,
            negative_inside: true,
            snap_centro_id: false,
        })
        .configure_sets(
            Update,
            (
                IsosurfaceExtractionSet::Sample,
                IsosurfaceExtractionSet::BuildOctree,
                IsosurfaceExtractionSet::Extract,
                IsosurfaceExtractionSet::Meshing,
            )
                .chain()
                .before(TerrainSystemSet::GenerateTerrain),
        )
        .add_plugins(SampleSurfacePlugin)
        .add_plugins(OctreePlugin)
        .add_plugins(ExtractPlugin)
        .add_plugins(MeshingPlugin);
    }
}
