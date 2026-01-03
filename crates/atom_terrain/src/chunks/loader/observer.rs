use std::ops::RangeInclusive;

use bevy::prelude::*;

/**
 * 观察者组件，标记实体为地形观察者。
 * 这主要用于实现地形的动态加载和卸载。
 * 目前假定总是挂在摄像机上，因为使用了摄像机的Frustum进行可见性判断。
 */
#[derive(Component, Debug, Default)]
#[require(Transform)]
pub struct TerrainObserver;

#[derive(Component, Debug)]
#[require(TerrainObserver)]
pub struct TerrainObserverConfig {
    /// 观察者的地形加载范围，单位为chunk数量。以观察者为中心。
    pub terrain_load_radius: u32,
    /// 观察者的地形加载高度范围，单位为chunk数量。以观察者为中心。
    pub terrain_height_range: RangeInclusive<i32>,
    /// 宽松边界，在实际加载范围之外额外加载的chunk数量，用于避免边界处的突然加载/卸载
    pub margin: u32,
}

impl Default for TerrainObserverConfig {
    fn default() -> Self {
        Self {
            terrain_load_radius: 2,
            terrain_height_range: -2..=2,
            margin: 1,
        }
    }
}

impl TerrainObserverConfig {
    pub fn new(terrain_load_radius: u32, terrain_height_range: RangeInclusive<i32>) -> Self {
        Self {
            terrain_load_radius,
            terrain_height_range,
            margin: 1,
        }
    }

    pub fn with_margin(mut self, margin: u32) -> Self {
        self.margin = margin;
        self
    }
}
