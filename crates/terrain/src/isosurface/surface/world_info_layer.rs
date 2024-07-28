use std::ops::{Index, Range};

/// 原始信息，没有受到外部影响。
use bevy::prelude::*;
use noise::{NoiseFn, Simplex};
use serde::{Deserialize, Serialize};
use settings::Setting;
use strum::EnumCount;

use super::csg::arc_noise::ArcNoise;

/// 湿度范围为-1到1, 0为正常，-1表示干燥，1表示湿润
#[derive(Resource)]
pub struct WorldHumidity {
    pub humidity: ArcNoise<f64, 3>,
}

impl WorldHumidity {
    fn new(seed: u32) -> Self {
        let simplex = Simplex::new(seed);

        Self {
            humidity: ArcNoise::new(simplex),
        }
    }

    fn get_humidity(&self, location: Vec3) -> f32 {
        self.humidity
            .get([location.x as f64, location.y as f64, location.z as f64]) as f32
    }
}

#[derive(Resource)]
pub struct WorldTemperature {
    // 0海拔高度温度
    pub temperature: ArcNoise<f64, 2>,
    /// 每海拔高度降低的温度
    pub temperature_decrease: f32,
}

impl WorldTemperature {
    pub fn new(seed: u32) -> Self {
        let simplex = Simplex::new(seed);

        Self {
            temperature: ArcNoise::new(simplex),
            temperature_decrease: 0.1,
        }
    }

    pub fn get_temperature(&self, location: Vec3) -> f32 {
        self.temperature.get([location.x as f64, location.z as f64]) as f32
            - self.temperature_decrease * location.y
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount, Serialize, Deserialize)]
pub enum TerrainType {
    Seabed,
    Plain,
    Hills,
    LowMountain,
    // 高原
    Plateau,
    Mountain,
    // 火山
    Volcano,
}

// 高度范围，湿度范围，温度范围，地形类型

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeSelector<T> {
    pub ranges: Vec<Range<f32>>,
    pub values: Vec<T>,
}

impl<T> RangeSelector<T> {
    pub fn new(ranges: Vec<Range<f32>>, values: Vec<T>) -> Self {
        Self { ranges, values }
    }

    pub fn get(&self, index: f32) -> Option<&T> {
        for (i, range) in self.ranges.iter().enumerate() {
            if range.contains(&index) {
                return Some(&self.values[i]);
            }
        }
        None
    }
}

impl<T> Index<f32> for RangeSelector<T> {
    type Output = T;

    fn index(&self, index: f32) -> &Self::Output {
        self.get(index).unwrap()
    }
}

#[derive(Setting, Resource, Debug, Clone, Serialize, Deserialize, TypePath, Asset)]
pub struct TerrainTypeSetting {
    pub terrain_selector: RangeSelector<RangeSelector<RangeSelector<TerrainType>>>,
}

impl TerrainTypeSetting {
    pub fn get_terrain_type(
        &self,
        height: f32,
        temperature: f32,
        humidity: f32,
    ) -> Option<TerrainType> {
        self.terrain_selector
            .get(height)
            .and_then(|x| x.get(temperature).and_then(|x| x.get(humidity).copied()))
    }
}

impl Default for TerrainTypeSetting {
    fn default() -> Self {
        Self {
            terrain_selector: RangeSelector {
                ranges: vec![Range {
                    start: 0.0,
                    end: 1.0,
                }],
                values: vec![RangeSelector {
                    ranges: vec![Range {
                        start: 0.0,
                        end: 1.0,
                    }],
                    values: vec![RangeSelector {
                        ranges: vec![Range {
                            start: 0.0,
                            end: 1.0,
                        }],
                        values: vec![
                            TerrainType::Seabed,
                            TerrainType::Plain,
                            TerrainType::Hills,
                            TerrainType::LowMountain,
                            TerrainType::Plateau,
                            TerrainType::Mountain,
                            TerrainType::Volcano,
                        ],
                    }],
                }],
            },
        }
    }
}
