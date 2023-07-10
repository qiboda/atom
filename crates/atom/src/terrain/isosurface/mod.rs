use bevy::prelude::*;
use meshing::MeshingPlugin;
use octree::OctreePlugin;
use sample::SampleSurfacePlugin;
use surface::{densy_function::Sphere, shape_surface::ShapeSurface};

pub mod cms;
pub mod meshing;
pub mod octree;
pub mod sample;
pub mod surface;

#[derive(PartialEq, Eq, Debug, Hash, Clone, SystemSet)]
pub enum IsosurfaceExtractionSet {
    Initialize,
    Sample,
    Extract,
    Meshing,
}

#[derive(Default, Component, Debug)]
pub struct IsosurfaceExtract;

#[derive(PartialEq, Eq, Debug, Clone, Component)]
pub enum IsosurfaceExtractionState {
    Initialize,
    Sample,
    Extract,
    Meshing,
}

#[derive(Bundle, Debug)]
struct IsosurfaceExtractionBundle {
    extract: IsosurfaceExtract,
    state: IsosurfaceExtractionState,
}

#[derive(Default, Debug)]
pub struct IsosurfaceExtractionPlugin;

impl Plugin for IsosurfaceExtractionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShapeSurface {
            densy_function: Box::new(Sphere),
            iso_level: Vec3::ZERO,
            negative_inside: true,
            snap_centro_id: true,
        })
        .configure_sets(
            Startup,
            (
                IsosurfaceExtractionSet::Initialize,
                IsosurfaceExtractionSet::Sample,
                IsosurfaceExtractionSet::Extract,
                IsosurfaceExtractionSet::Meshing,
            )
                .chain(),
        )
        .add_plugin(SampleSurfacePlugin)
        .add_plugin(OctreePlugin)
        .add_plugin(MeshingPlugin);
    }
}
