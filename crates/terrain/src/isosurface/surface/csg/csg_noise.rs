use std::fmt::Debug;

use bevy::math::Vec3;
use noise::{Add, Curve, Fbm, MultiFractal, NoiseFn, Perlin, Seedable, Turbulence};

use crate::isosurface::surface::csg::{arc_noise::ArcNoise, falloff_map::MapEdge};

use super::CSGNode;

pub struct WorldGenerator {
    pub random_seed: u32,
    pub height_noise: ArcNoise<f64, 2>,
}

impl Debug for WorldGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorldGenerator")
            .field("random_seed", &self.random_seed)
            .finish()
    }
}

impl CSGNode for WorldGenerator {
    fn eval(&self, location: &Vec3, value: &mut f32) {
        let x = location.x as f64 / 32768.0;
        let z = location.z as f64 / 32768.0;
        *value = location.y - self.height_noise.get([x, z]) as f32 * 1000.0
    }
}

pub mod reference;

impl WorldGenerator {
    pub fn new(_x: f32) -> Self {
        // Frequency-周期性过程的特征等于每单位时间重复或事件（操作）发生的次数。
        // Lacunarity-控制频率的变化。
        // Persistence-控制振幅的变化。
        // Octave-具有不同频率的样本数量。
        // 因此振幅越来越高，频率越来越高。
        // let perlin_fbm = Fbm::<Perlin>::new(32)
        //     .set_frequency(0.0000003)
        //     .set_lacunarity(10.0)
        //     .set_persistence(100.0)
        //     .set_octaves(5);

        let map_edge = MapEdge::new()
            .with_half_size([1.0, 1.0])
            .with_inner_half_size([0.5, 0.5])
            .with_a_b(3.0, 2.0);

        // let noise_map = PlaneMapBuilder::new_fn(|x| map_edge.get(x))
        //     .set_size(1024, 1024)
        //     .set_x_bounds(-1.0, 1.0)
        //     .set_y_bounds(-1.0, 1.0)
        //     .build();
        // noise_map.write_to_file(Path::new("map_edge.png"));

        let base_continent_def_fb0 = Fbm::<Perlin>::new(23)
            .set_frequency(1.0)
            .set_persistence(0.5)
            .set_lacunarity(3.0)
            .set_octaves(3);

        let base_continent_def_cu = Curve::new(base_continent_def_fb0)
            .add_control_point(-1.0000, -0.125)
            .add_control_point(0.0000, 0.0)
            .add_control_point(0.0625, 0.125)
            .add_control_point(0.1250, 0.250)
            .add_control_point(0.2500, 1.000)
            .add_control_point(0.5000, 0.250)
            .add_control_point(0.7500, 0.250)
            .add_control_point(1.0000, 0.500);

        let add = Add::new(base_continent_def_cu, map_edge);

        let continent_def_tu0 = Turbulence::<_, Perlin>::new(add)
            .set_seed(23 + 10)
            .set_frequency(1.0 * 55.25)
            .set_power(1.0 / 113.75)
            .set_roughness(30);

        // let noise_map = PlaneMapBuilder::new_fn(|x| continent_def_tu0.get(x))
        //     .set_size(1024, 1024)
        //     .set_x_bounds(-1.0, 1.0)
        //     .set_y_bounds(-1.0, 1.0)
        //     .build();
        // noise_map.write_to_file(Path::new("noise.png"));

        Self {
            random_seed: 32,
            height_noise: ArcNoise::new(continent_def_tu0),
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_falloff_map() {
        // let mut pixels: Vec<u8> = Vec::with_capacity(SIZE * SIZE);

        // for i in &map {
        //     for j in i {
        //         pixels.push((j.clamp(0.0, 1.0) * 255.0) as u8);
        //     }
        // }

        // let _ = image::save_buffer(
        //     Path::new("falloff.png"),
        //     &pixels,
        //     SIZE as u32,
        //     SIZE as u32,
        //     image::ColorType::L8,
        // );
    }
}
