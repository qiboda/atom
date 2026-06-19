use std::ops::RangeInclusive;

use bevy::prelude::*;

/// 地形观察者组件。
///
/// `TerrainObserver` 标记实体为地形加载中心（通常是摄像机）。
/// `TerrainObserverConfig` 控制加载范围：
/// - `terrain_load_radius`：水平加载半径（chunk 单位）
/// - `terrain_height_range`：垂直加载范围（相对于观察者 chunk Y 的偏移）
/// - `margin`：卸载宽松边界，防止边界抖动

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
