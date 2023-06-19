#![allow(dead_code)]
#![allow(unused_variables)]

use std::{path::Path, rc::Rc};

use bevy::{log::LogPlugin, prelude::App};
use cms::CMS;
use densy_function::Sphere;
use mesh::Mesh;
use nalgebra::Vector3;
use shape_surface::ShapeSurface;

pub mod address;
pub mod cms;
pub mod densy_function;
pub mod iso_surface;
pub mod mesh;
pub mod octree;
pub mod sample;
pub mod shape_surface;

const BBOX_SIZE: f32 = 2.0;

const MIN_OCTREE_RES: usize = 2;
const MAX_OCTREE_RES: usize = 8;

const COMPLEX_SURFACE_THRESHOLD: f32 = 0.85;

const ADDRESS_SIZE: usize = MAX_OCTREE_RES;

fn main() {
    let mut app = App::new();

    app.add_plugin(LogPlugin::default()).run();

    let sphere = Rc::new(Sphere);
    let shape_surface = Rc::new(ShapeSurface {
        shape: sphere,
        iso_level: 0.0,
        negative_inside: true,
    });

    let container = Vector3::new(
        (-BBOX_SIZE, BBOX_SIZE),
        (-BBOX_SIZE, BBOX_SIZE),
        (-BBOX_SIZE, BBOX_SIZE),
    );

    let mut cms = CMS::new(0.0, container, shape_surface.clone());

    cms.initialize();

    // return;

    let mut mesh = Mesh {
        vertices: Vec::new(),
        indices: Vec::new(),
    };
    cms.extract_surface(&mut mesh);

    mesh.export_obj(Path::new("sphere.obj"));
}
