use std::{fmt::Debug, path::Path};

use bevy::{math::Vec3A, prelude::Vec3};
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, NoiseFn, Perlin,
};

pub trait DensityFunction: Sync + Send {
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
        (x - 8.0) * (x - 8.0) + (y - 8.0) * (y - 8.0) + (z - 8.0) * (z - 8.0) - 8.0
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
        y
    }
}

#[derive(Default, Debug)]
pub struct Cube;

pub fn cube(b: Vec3A, p: Vec3A) -> f32 {
    let q = p.abs() - b;
    q.max(Vec3A::ZERO).length() + q.max_element().min(0.0)
}

impl DensityFunction for Cube {
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        cube(Vec3A::splat(10.0), Vec3A::new(x - 8.0, y - 8.0, z - 8.0))
    }
}

#[derive(Default)]
pub struct NoiseSurface {
    pub random_seed: u32,
    pub perlin_fbm: Fbm<Perlin>,
}

impl NoiseSurface {
    pub fn new() -> Self {
        // Frequency-周期性过程的特征等于每单位时间重复或事件（操作）发生的次数。
        // Lacunarity-控制频率的变化。
        // Persistence-控制振幅的变化。
        // Octave-具有不同频率的样本数量。
        let perlin_fbm = Fbm::<Perlin>::new(32)
            .set_frequency(0.0003)
            .set_lacunarity(3.0)
            .set_persistence(16.0)
            .set_octaves(3);

        let noise_map = PlaneMapBuilder::new(perlin_fbm.clone())
            .set_size(1000, 1000)
            .set_x_bounds(-1000.0, 1000.0)
            .set_y_bounds(-1000.0, 1000.0)
            .build();
        noise_map.write_to_file(Path::new("noise.png"));

        Self {
            random_seed: 32,
            perlin_fbm,
        }
    }
}

impl DensityFunction for NoiseSurface {
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        y - self.perlin_fbm.get([x as f64, z as f64]) as f32 * 30.0 + 0.20
    }
}
