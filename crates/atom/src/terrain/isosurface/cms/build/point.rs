use bevy::prelude::Vec3;

#[derive(Debug, Default)]
pub struct Point {
    // position instead of coord
    position: Vec3,
    value: f32,
}

impl Point {
    pub fn new() -> Self {
        Self {
            position: Vec3::new(1.0, 0.0, 0.0),
            value: 10.0,
        }
    }

    pub fn new_with_position_and_value(position: Vec3, value: f32) -> Self {
        Self { position, value }
    }
}

impl Point {
    pub fn get_position(&self) -> &Vec3 {
        &self.position
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }

    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }
}
