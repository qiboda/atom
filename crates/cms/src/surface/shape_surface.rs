use bevy::prelude::*;

use super::densy_function::DensyFunction;

#[derive(Resource)]
pub struct ShapeSurface {
    pub densy_function: Box<dyn DensyFunction>,

    pub iso_level: f32,

    pub negative_inside: bool,
}

impl ShapeSurface {
    pub fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        self.densy_function.get_value(x, y, z)
    }

    pub fn set_iso_level(&mut self, iso_level: f32) {
        self.iso_level = iso_level;
    }

    pub fn get_iso_level(&self) -> f32 {
        self.iso_level
    }

    pub fn set_negative_inside(&mut self, negative_inside: bool) {
        self.negative_inside = negative_inside;
    }

    pub fn is_negative_inside(&self) -> bool {
        self.negative_inside
    }
}
