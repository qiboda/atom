pub trait DensyFunction {
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32;
}

pub struct Sphere;

impl DensyFunction for Sphere {
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        x * x + y * y + z * z - 1.0
    }
}

// 圆环面
pub struct Torus;

impl DensyFunction for Torus {
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        let r_outer = 0.45;
        let r_inner = 0.2;
        let x0 = x - 0.25;
        ((x0 * x0 + y * y + z * z + r_outer * r_outer - r_inner * r_inner)
            * (x0 * x0 + y * y + z * z + r_outer * r_outer - r_inner * r_inner))
            - (4.0 * r_outer * r_outer) * (z * z + x0 * x0)
    }
}

pub struct Cube;

impl DensyFunction for Cube {
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        (x.abs() - 1.0).max((y.abs() - 1.0).max(z.abs() - 1.0))
    }
}
