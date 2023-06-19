use std::rc::Rc;

use crate::{densy_function::DensyFunction, iso_surface::IsoSurface};

#[derive(Debug)]
pub struct ShapeSurface {
    pub shape: Rc<dyn DensyFunction>,

    pub iso_level: f32,

    pub negative_inside: bool,
}

impl IsoSurface for ShapeSurface {
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        self.shape.get_value(x, y, z)
    }

    fn set_iso_level(&mut self, iso_level: f32) {
        self.iso_level = iso_level;
    }

    fn get_iso_level(&self) -> f32 {
        self.iso_level
    }

    fn set_negative_inside(&mut self, negative_inside: bool) {
        self.negative_inside = negative_inside;
    }

    fn is_negative_inside(&self) -> bool {
        self.negative_inside
    }
}
