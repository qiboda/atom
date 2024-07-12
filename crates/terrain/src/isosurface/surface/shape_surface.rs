use std::sync::{Arc, RwLock};

use bevy::prelude::*;

use crate::isosurface::dc::octree::OctreeSampler;

use super::density_function::DensityFunction;

#[derive(Resource, Debug)]
pub struct IsosurfaceContext {
    pub shape_surface: Arc<RwLock<ShapeSurface>>,
}

#[derive(Debug)]
pub struct ShapeSurface {
    pub density_function: Box<dyn DensityFunction>,

    pub iso_level: Vec3,
}

impl ShapeSurface {
    #[inline]
    pub fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        self.density_function.get_value(x, y, z)
    }

    #[inline]
    pub fn get_value_from_vec(&self, pos: Vec3) -> f32 {
        self.get_value(pos.x, pos.y, pos.z)
    }

    #[inline]
    pub fn set_iso_level(&mut self, iso_level: Vec3) {
        self.iso_level = iso_level;
    }

    #[inline]
    pub fn get_iso_level(&self) -> Vec3 {
        self.iso_level
    }
}

impl ShapeSurface {
    pub fn get_range_values(&self, offset: Vec3, size: Vec3, grain_size: Vec3) -> Vec<f32> {
        self.density_function
            .get_range_values(offset, size, grain_size)
    }
}

impl<'a> OctreeSampler for std::sync::RwLockReadGuard<'a, ShapeSurface> {
    fn sampler(&self, loc: Vec3) -> f32 {
        self.get_value_from_vec(loc)
    }

    fn sampler_split(&self, x: f32, y: f32, z: f32) -> f32 {
        self.get_value(x, y, z)
    }
}
