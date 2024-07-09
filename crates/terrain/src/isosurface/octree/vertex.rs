use bevy::prelude::Vec3;

#[derive(Debug, Default)]
pub struct Vertex {
    // position instead of coord
    pub position: Vec3,
    pub value: f32,
}

impl Vertex {
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
