use std::sync::{Arc, RwLock};

use bevy::prelude::*;

// use crate::isosurface::{dc::octree::OctreeSampler, surface::csg::csg_shapes::CSGNone};

use super::csg::{apply_csg_operation, csg_shapes::CSGNone, CSGNode, CSGOperation};

#[derive(Resource, Debug)]
pub struct IsosurfaceContext {
    pub shape_surface: Arc<RwLock<ShapeSurface>>,
}

#[derive(Debug)]
pub struct ShapeSurface {
    csg_root: Box<dyn CSGNode>,
}

impl ShapeSurface {
    pub fn new(csg_root: Box<dyn CSGNode>) -> Self {
        Self { csg_root }
    }
}

impl ShapeSurface {
    #[inline]
    pub fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        self.get_value_from_vec(&Vec3::new(x, y, z))
    }

    #[inline]
    pub fn get_value_from_vec(&self, location: &Vec3) -> f32 {
        let mut value = 0.0;
        self.csg_root.eval(location, &mut value);
        value
    }
}

impl ShapeSurface {
    pub fn apply_csg_operation(&mut self, new_node: Box<dyn CSGNode>, operation: CSGOperation) {
        let root_node = std::mem::replace(&mut self.csg_root, Box::new(CSGNone));
        self.csg_root = apply_csg_operation(root_node, new_node, operation);
    }
}

// impl<'a> OctreeSampler for std::sync::RwLockReadGuard<'a, ShapeSurface> {
//     fn sampler(&self, loc: Vec3) -> f32 {
//         self.get_value_from_vec(&loc)
//     }

//     fn sampler_split(&self, x: f32, y: f32, z: f32) -> f32 {
//         self.get_value(x, y, z)
//     }
// }
