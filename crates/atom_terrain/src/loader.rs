//! 地形动态加载系统。
//!
//! 每帧根据观察者（`TerrainObserver`）位置加载/卸载 chunk，通过消息队列驱动 GPU compute 管线。

use bevy::prelude::*;

use crate::{
    chunk::{ChunkLoadMsg, ChunkUnloadMsg, TerrainChunkCoord, TerrainLoadedChunks},
    setting::TerrainSetting,
};

/// 观察者标记组件，挂载到玩家相机等需要加载地形 chunk 的实体上
#[derive(Component, Default)]
#[require(Transform)]
pub struct TerrainObserver;

/// 观察者配置，控制加载半径与高度范围
#[derive(Component)]
#[require(TerrainObserver)]
pub struct TerrainObserverConfig {
    /// 以 chunk 为单位的水平加载半径
    pub load_radius: u32,
    /// 垂直加载范围（相对于观察者 chunk Y）
    pub height_range: std::ops::RangeInclusive<i32>,
    /// 卸载宽松边界（chunk 单位），防止边界抖动
    pub margin: u32,
}

impl Default for TerrainObserverConfig {
    fn default() -> Self {
        Self { load_radius: 3, height_range: -2..=2, margin: 1 }
    }
}

/// 每帧根据观察者位置加载/卸载 chunk
pub fn update_grid_chunks(
    mut commands: Commands,
    observers: Query<(&GlobalTransform, &TerrainObserverConfig), With<TerrainObserver>>,
    terrain_setting: Res<TerrainSetting>,
    mut loaded: ResMut<TerrainLoadedChunks>,
    mut load_tx: MessageWriter<ChunkLoadMsg>,
    mut unload_tx: MessageWriter<ChunkUnloadMsg>,
) {
    let chunk_size = terrain_setting.chunk_size();
    let mut keep: Vec<TerrainChunkCoord> = Vec::new();

    for (transform, config) in observers.iter() {
        let center = transform.translation();
        let center_chunk = TerrainChunkCoord::from_world(center, chunk_size);
        let r = config.load_radius as i32;
        let margin = config.margin as i32;
        let unload_r = r + margin;

        for y in center_chunk.0.y + *config.height_range.start()..=center_chunk.0.y + *config.height_range.end() {
            for x in center_chunk.0.x - unload_r..=center_chunk.0.x + unload_r {
                for z in center_chunk.0.z - unload_r..=center_chunk.0.z + unload_r {
                    let coord = TerrainChunkCoord::new(x, y, z);
                    keep.push(coord);
                    if !loaded.contains(&coord) && x.abs() <= r && z.abs() <= r {
                        load_tx.write(ChunkLoadMsg { coord });
                    }
                }
            }
        }
    }

    let to_unload: Vec<TerrainChunkCoord> = loaded
        .iter()
        .filter(|(c, _)| !keep.contains(c))
        .map(|(c, _)| *c)
        .collect();

    for coord in to_unload {
        if let Some(entity) = loaded.remove(&coord) {
            commands.entity(entity).despawn();
            unload_tx.write(ChunkUnloadMsg { coord });
        }
    }
}
