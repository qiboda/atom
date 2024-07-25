use bevy::math::Vec3;

use super::CSGNode;

#[derive(Debug)]
pub struct CSGSphere {
    pub position: Vec3,
    pub radius: f32,
}

impl CSGNode for CSGSphere {
    fn eval(&self, point: &Vec3, value: &mut f32) {
        let diff = *point - self.position;
        *value = diff.length() - self.radius;
    }
}

#[derive(Debug)]
pub struct CSGCylinder {
    pub position: Vec3,
    pub direction: Vec3,
    pub radius: f32,
}

impl CSGNode for CSGCylinder {
    fn eval(&self, point: &Vec3, value: &mut f32) {
        let diff = *point - self.position;
        let d = diff.dot(self.direction);
        let p = self.position + self.direction * d;
        let r = (p - *point).length() - self.radius;

        *value = r;
    }
}

#[derive(Debug)]
pub struct CSGTorus {
    pub position: Vec3,
    pub radius: f32,
    pub thickness: f32,
}

impl CSGNode for CSGTorus {
    fn eval(&self, point: &Vec3, value: &mut f32) {
        let diff = *point - self.position;
        let x = diff.x;
        let z = diff.z;
        let y = diff.y;

        let r = (x * x + z * z).sqrt() - self.radius;
        let d = (r * r + y * y).sqrt() - self.thickness;

        *value = d;
    }
}

#[derive(Default, Debug)]
pub struct CSGCube {
    pub location: Vec3,
    pub half_size: Vec3,
}

fn cube(position: Vec3, half_size: Vec3) -> f32 {
    let q = position.abs() - half_size;
    q.max(Vec3::ZERO).length() + q.max_element().min(0.0)
}

impl CSGNode for CSGCube {
    fn eval(&self, point: &Vec3, value: &mut f32) {
        let position = *point - self.location;
        *value = cube(position, self.half_size)
    }
}

fn plane(location: Vec3, normal: Vec3, height: f32) -> f32 {
    // n must be normalized
    location.dot(normal) + height
}

#[derive(Debug)]
pub struct CSGPanel {
    pub location: Vec3,
    pub normal: Vec3,
    pub height: f32,
}

impl CSGNode for CSGPanel {
    fn eval(&self, point: &Vec3, value: &mut f32) {
        let position = *point - self.location;
        *value = plane(position, self.normal, self.height);
    }
}

#[derive(Debug)]
pub struct CSGNone;

impl CSGNode for CSGNone {
    fn eval(&self, _point: &Vec3, _value: &mut f32) {}
}
