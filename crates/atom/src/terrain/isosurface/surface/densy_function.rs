use std::{
    arch::x86_64::{__m128, _mm_cvtss_f32, _mm_set1_ps},
    fmt::Debug,
};

use bevy::prelude::Vec3;
use simdnoise::NoiseBuilder;

pub trait DensyFunction: Sync + Send + Debug {
    // from world position
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32;

    // from world offset, and world size, and grain size
    fn get_range_values(&self, offset: Vec3, size: Vec3, grain_size: Vec3) -> Vec<f32> {
        let mut vec = Vec::with_capacity((size.x * size.y * size.z) as usize);
        let sample_num = (size / grain_size).as_uvec3();
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

impl DensyFunction for Sphere {
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        x * x + y * y + z * z - 1.0
    }
}

// 圆环面
#[derive(Default, Debug)]
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

#[derive(Default, Debug)]
pub struct Cube;

impl DensyFunction for Cube {
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        (x.abs() - 1.0).max((y.abs() - 1.0).max(z.abs() - 1.0))
    }
}

#[derive(Debug, Default)]
pub struct NoiseSurface {
    pub seed: i32,
    pub frequency: f32,
    pub lacunarity: f32,
    pub gain: f32,
    pub octaves: u8,
}

impl DensyFunction for NoiseSurface {
    fn get_range_values(&self, offset: Vec3, size: Vec3, grain_size: Vec3) -> Vec<f32> {
        let div_grain_size = 1.0 / grain_size;
        let (values, _min, _max) = NoiseBuilder::fbm_3d_offset(
            offset.x * div_grain_size.x,
            (size.x * div_grain_size.x) as usize,
            offset.y * div_grain_size.y,
            (size.y * div_grain_size.y) as usize,
            offset.z * div_grain_size.z,
            (size.z * div_grain_size.z) as usize,
        )
        .with_seed(self.seed)
        .with_freq(self.frequency)
        .with_gain(self.gain)
        .with_lacunarity(self.lacunarity)
        .with_octaves(self.octaves)
        .generate();

        values
    }

    // todo: fix without freq
    fn get_value(&self, x: f32, y: f32, z: f32) -> f32 {
        assert!(false);
        unsafe {
            // avx2 turbulence
            let x = _mm_set1_ps(x);
            let y = _mm_set1_ps(y);
            let z = _mm_set1_ps(z);
            let _freq = _mm_set1_ps(self.frequency);
            let lacunarity = _mm_set1_ps(self.lacunarity);
            let gain = _mm_set1_ps(2.0);
            let value_m128: __m128 = simdnoise::intrinsics::sse2::fbm_3d(
                x,
                y,
                z,
                lacunarity,
                gain,
                self.octaves,
                self.seed,
            );
            _mm_cvtss_f32(value_m128)
        }
    }
}
