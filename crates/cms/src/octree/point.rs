use nalgebra::Vector3;

pub struct Point {
    // position instead of coord
    position: Vector3<f32>,
    value: f32,
    gradient: Vector3<f32>,
}

impl Point {
    pub fn new() -> Self {
        Self {
            position: Vector3::new(1.0, 0.0, 0.0),
            value: 10.0,
            gradient: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn new_with_position_and_value(position: &Vector3<f32>, value: f32) -> Self {
        Self {
            position: position.clone(),
            value,
            gradient: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn new_with_position_and_value_and_gradient(
        position: &Vector3<f32>,
        value: f32,
        gradient: &Vector3<f32>,
    ) -> Self {
        Self {
            position: position.clone(),
            value,
            gradient: gradient.clone(),
        }
    }
}

impl Point {
    pub fn get_position(&self) -> &Vector3<f32> {
        &self.position
    }

    pub fn get_value(&self) -> f32 {
        self.value
    }

    pub fn get_gradient(&self) -> &Vector3<f32> {
        &self.gradient
    }

    pub fn set_position(&mut self, position: Vector3<f32>) {
        self.position = position;
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    pub fn set_gradient(&mut self, gradient: Vector3<f32>) {
        self.gradient = gradient;
    }
}
