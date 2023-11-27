use std::fmt::Debug;

use bevy::prelude::{Vec2, Vec3};

pub trait DensityFunction: Sync + Send + Debug {
    // from world position
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32;

    // from world offset, and range size, and grain size
    fn get_range_values(&self, offset: Vec3, size: Vec3, grain_size: Vec3) -> Vec<f32> {
        let mut vec = Vec::with_capacity((size.x * size.y * size.z) as usize);
        let sample_num = (size / grain_size).round().as_uvec3();
        for x in 0..sample_num.x {
            for y in 0..sample_num.y {
                for z in 0..sample_num.z {
                    let value = self.get_value(
                        offset.x + x as f32 * grain_size.x,
                        offset.y + y as f32 * grain_size.y,
                        offset.z + z as f32 * grain_size.z,
                    );
                    vec.push(value);
                }
            }
        }
        vec
    }
}

#[derive(Default, Debug)]
pub struct Sphere;

impl DensityFunction for Sphere {
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        x * x + y * y + z * z - 32.0
    }
}

// 圆环面
#[derive(Default, Debug)]
pub struct Torus;

impl DensityFunction for Torus {
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        let r_outer = 4.0;
        let r_inner = 2.0;
        let x0 = x - 2.0;
        ((x0 * x0 + y * y + z * z + r_outer * r_outer - r_inner * r_inner)
            * (x0 * x0 + y * y + z * z + r_outer * r_outer - r_inner * r_inner))
            - (4.0 * r_outer * r_outer) * (z * z + x0 * x0)
    }
}

// 圆环面
#[derive(Default, Debug)]
pub struct Panel;

impl DensityFunction for Panel {
    fn get_value(&self, _x: f32, y: f32, _z: f32) -> f32 {
        match y {
            y if y < 0.0 => -0.5,
            y if y > 0.0 => 0.5,
            _ => y,
        }
    }
}

#[derive(Default, Debug)]
pub struct Cube;

impl DensityFunction for Cube {
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        (x.abs() - 4.0).max((y.abs() - 4.0).max(z.abs() - 4.0))
    }
}

#[derive(Debug, Default)]
pub struct NoiseSurface {
    pub frequency: f32,
    pub lacunarity: f32,
    pub gain: f32,
    pub octaves: usize,
}

impl DensityFunction for NoiseSurface {
    // todo: fix without freq
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        // return y;
        y - noisy_bevy::fbm_simplex_2d(
            Vec2::new(x, z) * self.frequency,
            self.octaves,
            self.lacunarity,
            self.gain,
        )
        .abs()
    }
}
