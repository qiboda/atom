use bevy::prelude::*;

/**
 * 观察者组件，标记实体为地形观察者。
 * 这主要用于实现地形的动态加载和卸载。
 */
#[derive(Component, Debug, Default)]
pub struct TerrainObserver;

#[derive(Component, Debug, Default)]
#[require(TerrainObserver)]
pub struct TerrainObserverConfig {
    /// 观察者的地形加载范围，单位为米。
    /// 如果没有设置，则使用摄像机的可视距离。
    pub terrain_load_radius: Option<f32>,
}
