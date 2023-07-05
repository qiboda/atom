use bevy::prelude::*;
use mesh::MeshingPlugin;
use octree::OctreePlugin;
use sample::SampleSurfacePlugin;
use surface::{densy_function::Sphere, shape_surface::ShapeSurface};

// #![allow(dead_code)]
// #![allow(unused_variables)]
//

/// shape surface
///   cms plugin cms as a empty bundle and has these children.
///      => sample data as cache..
///      => octree => build
///      => meshing => build
pub mod cms;
pub mod mesh;
pub mod octree;
pub mod sample;
pub mod surface;

#[derive(Default, Component)]
struct IsosurfaceExtract;

#[derive(Default, Debug)]
pub struct IsosurfaceExtractionPlugin;

#[derive(PartialEq, Eq, Debug, Hash, Clone, SystemSet)]
pub enum IsosurfaceExtractionSet {
    Initialize,
    Sample,
    Extract,
    Meshing,
}

/// todo: 坐标变换，从密度函数的坐标系到采样坐标系
impl Plugin for IsosurfaceExtractionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ShapeSurface {
            densy_function: Box::new(Sphere),
            iso_level: Vec3::ZERO,
            negative_inside: true,
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

//
// use std::{path::Path, rc::Rc};
//
// use bevy::{
//     log::LogPlugin,
//     prelude::{info, App},
// };
// use mesh::Mesh;
// use nalgebra::Vector3;
//
// pub mod cms;
// pub mod mesh;
// pub mod octree;
// pub mod sample;
// pub mod surface;
//
// const BBOX_SIZE: f32 = 2.0;
//
// const MIN_OCTREE_RES: usize = 2;
// const MAX_OCTREE_RES: usize = 8;
//
// const COMPLEX_SURFACE_THRESHOLD: f32 = 0.85;
//
// const ADDRESS_SIZE: usize = MAX_OCTREE_RES;
//
// fn main() {
//     let mut app = App::new();
//
//     app.add_plugin(LogPlugin::default()).run();
//
//     info!("start");
//
//     // let sphere = Rc::new(Sphere);
//     // let sphere = Rc::new(Torus);
//     let sphere = Rc::new(Cube);
//     let shape_surface = Rc::new(ShapeSurface {
//         shape: sphere,
//         iso_level: 0.0,
//         negative_inside: true,
//     });
//
//     let container = Vector3::new(
//         (-BBOX_SIZE, BBOX_SIZE),
//         (-BBOX_SIZE, BBOX_SIZE),
//         (-BBOX_SIZE, BBOX_SIZE),
//     );
//
//     let mut cms = CMS::new(0.0, container, shape_surface.clone());
//
//     cms.initialize();
//
//     // return;
//
//     let mut mesh = Mesh {
//         vertices: Vec::new(),
//         indices: Vec::new(),
//     };
//     cms.extract_surface(&mut mesh);
//
//     mesh.export_obj(Path::new("sphere.obj"));
//
//     info!("end");
// }
