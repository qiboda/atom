use nalgebra::Vector3;

#[derive(Debug)]
pub struct Vertex {
    position: Vector3<f32>,
    normal: Vector3<f32>,
}

impl Vertex {
    pub fn new() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            normal: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn new_with_position_and_normal(position: &Vector3<f32>, normal: &Vector3<f32>) -> Self {
        Self {
            position: *position,
            normal: *normal,
        }
    }
}

impl Vertex {
    pub fn get_position(&self) -> &Vector3<f32> {
        &self.position
    }

    pub fn get_normal(&self) -> &Vector3<f32> {
        &self.normal
    }

    pub fn set_position(&mut self, position: &Vector3<f32>) {
        self.position = *position;
    }

    pub fn set_normal(&mut self, normal: &Vector3<f32>) {
        self.normal = *normal;
    }
}
