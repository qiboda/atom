#![allow(dead_code)]
#![allow(unused_variables)]

pub mod cms;
pub mod mesh;
pub mod octree;
pub mod sample;
pub mod surface;

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
