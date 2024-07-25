use std::path::Path;

use bevy::math::Vec3;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, NoiseFn, Perlin,
};

use super::CSGNode;

#[derive(Default, Debug, Clone)]
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
            .set_persistence(300.0)
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

impl CSGNode for NoiseSurface {
    fn eval(&self, location: &Vec3, value: &mut f32) {
        *value =
            location.y - self.perlin_fbm.get([location.x as f64, location.z as f64]) as f32 * 300.0
    }
}
