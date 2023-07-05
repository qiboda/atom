use bevy::prelude::Vec3;

#[derive(Debug)]
pub struct Point {
    // position instead of coord
    position: Vec3,
    value: f32,
    gradient: Vec3,
}

impl Point {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(1.0, 0.0, 0.0),
            value: 10.0,
            gradient: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn new_with_position_and_value(position: Vec3, value: f32) -> Self {
        Self {
            position: position.clone(),
            value,
            gradient: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn new_with_position_and_value_and_gradient(
        position: &Vec3,
        value: f32,
        gradient: &Vec3,
    ) -> Self {
        Self {
            position: position.clone(),
            value,
            gradient: gradient.clone(),
        }
    }
}

impl Point {
    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }

    pub fn get_gradient(&self) -> &Vec3 {
        &self.gradient
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    pub fn set_gradient(&mut self, gradient: Vec3) {
        self.gradient = gradient;
    }
}
