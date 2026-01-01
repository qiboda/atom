use std::ops::RangeInclusive;

use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    chunks::{
        chunk::{TerrainChunk, TerrainChunkCoord},
        loader::{
            loaded_chunks::{TerrainChunkLoadMsg, TerrainChunkUnloadMsg, TerrainLoadedChunks},
            observer::{TerrainObserver, TerrainObserverConfig},
        },
    },
    terrain::setting::TerrainSetting,
};

fn get_terrain_height_range(
    terrain_setting: &TerrainSetting,
    observer_config: &TerrainObserverConfig,
) -> RangeInclusive<i32> {
    let terrain_height_range = &terrain_setting.size_setting.height_range;
    let observer_height_range = &observer_config.terrain_height_range;

    let start = terrain_height_range
        .start()
        .max(observer_height_range.start());
    let end = terrain_height_range.end().min(observer_height_range.end());

    *start..=*end
}

/// grid chunk loader 主系统
pub fn update_grid_chunks(
    mut commands: Commands,
    observers: Query<(&GlobalTransform, &TerrainObserverConfig), With<TerrainObserver>>,
    terrain_setting: Res<TerrainSetting>,
    mut loaded_chunks: ResMut<TerrainLoadedChunks>,
    mut unload_requests: MessageWriter<TerrainChunkUnloadMsg>,
    mut load_requests: MessageWriter<TerrainChunkLoadMsg>,
) {
    if observers.is_empty() {
        error!("没有找到任何 TerrainObserver，无法加载地形！");
        return;
    }

    // TODO 使用第一个观察者的位置（可以扩展为支持多个观察者）
    let (observer_transform, observer_config) = observers.iter().next().expect("至少有一个观察者");

    let chunk_size = terrain_setting.chunk_setting.get_chunk_size();
    let visibility_radius = observer_config.terrain_load_radius;

    let height_range = get_terrain_height_range(&terrain_setting, observer_config);

    // 计算观察者所在的 chunk 坐标
    let observer_pos = observer_transform.translation();

    let observer_chunk_coord = TerrainChunkCoord::from_world_pos(observer_pos, chunk_size);
    let aa_chunk_coord = observer_chunk_coord
        - TerrainChunkCoord::new(visibility_radius as i32, 0, visibility_radius as i32);
    let bb_chunk_coord = observer_chunk_coord
        + TerrainChunkCoord::new(visibility_radius as i32, 0, visibility_radius as i32);

    // 计算需要加载的 chunks
    let mut desired_chunks: HashSet<TerrainChunkCoord> = HashSet::new();
    for x in aa_chunk_coord.x()..=bb_chunk_coord.x() {
        for z in aa_chunk_coord.z()..=bb_chunk_coord.z() {
            for dy in height_range.clone() {
                let chunk_y = observer_chunk_coord.y() + dy;
                let coord = TerrainChunkCoord::new(x, chunk_y, z);
                desired_chunks.insert(coord);
            }
        }
    }

    trace!(
        "Desired Chunks Count: {:?} -> {:?}",
        desired_chunks.len(),
        desired_chunks
    );

    // 找出需要卸载的 chunks
    let mut chunks_to_unload = Vec::new();
    for (coord, _entity) in loaded_chunks.iter() {
        if !desired_chunks.contains(coord) {
            chunks_to_unload.push(*coord);
        }
    }

    trace!(
        "Chunks to Unload Count: {:?} -> {:?}",
        chunks_to_unload.len(),
        chunks_to_unload
    );

    // 找出需要加载的 chunks
    let mut chunks_to_load = Vec::new();
    for coord in desired_chunks {
        if !loaded_chunks.contains(&coord) {
            chunks_to_load.push(coord);
        }
    }

    trace!(
        "Chunks to Load Count: {:?} -> {:?}",
        chunks_to_load.len(),
        chunks_to_load
    );

    // 卸载 chunks
    for coord in chunks_to_unload {
        if let Some(entity) = loaded_chunks.remove(&coord) {
            commands.entity(entity).despawn();
            unload_requests.write(TerrainChunkUnloadMsg { coord });
        }
    }

    // 加载 chunks
    for coord in chunks_to_load {
        let entity = commands
            .spawn((TerrainChunk, coord, Name::new(format!("Chunk_{:?}", coord))))
            .id();
        loaded_chunks.insert(coord, entity);

        load_requests.write(TerrainChunkLoadMsg { coord });
    }
}
