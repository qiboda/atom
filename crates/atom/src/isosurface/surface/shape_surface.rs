use bevy::prelude::*;

use super::densy_function::DensyFunction;

#[derive(Resource, Debug)]
pub struct ShapeSurface {
    pub densy_function: Box<dyn DensyFunction>,

    pub iso_level: Vec3,

    pub negative_inside: bool,
}

impl ShapeSurface {
    #[inline]
    pub fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        self.densy_function.get_value(x, y, z)
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

    #[inline]
    pub fn set_negative_inside(&mut self, negative_inside: bool) {
        self.negative_inside = negative_inside;
    }

    #[inline]
    pub fn is_negative_inside(&self) -> bool {
        self.negative_inside
    }
}
