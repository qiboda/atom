/// 地形 chunk 加载/卸载系统。
///
/// 核心逻辑：以 TerrainObserver（通常挂在摄像机上）为中心，
/// 在 `terrain_load_radius` 水平范围 + `terrain_height_range` 垂直范围内加载 chunk，
/// 在 `margin` 扩容后的范围外卸载 chunk。
///
/// - `get_terrain_height_range`：全局 height_range 与 observer 配置的交集
/// - `update_grid_chunks`：每帧计算需要加载/卸载的 chunk 列表并发送消息
use std::ops::RangeInclusive;

use bevy::{platform::collections::HashSet, prelude::*};

use crate::{
    chunks::{
        chunk::{TerrainChunk, TerrainChunkCoord},
        loader::{
            loaded_chunks::{TerrainChunkLoadMsg, TerrainChunkUnloadMsg, TerrainLoadedChunks},
            observer::{TerrainObserver, TerrainObserverConfig},
        },
        mesh::{
            visual::TerrainChunkVisual,
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
    let margin = observer_config.margin;

    let height_range = get_terrain_height_range(&terrain_setting, observer_config);

    // 计算观察者所在的 chunk 坐标
    let observer_pos = observer_transform.translation();

    let observer_chunk_coord = TerrainChunkCoord::from_world_pos(observer_pos, chunk_size);

    // 加载范围：只使用 visibility_radius
    let load_radius = visibility_radius as i32;
    let load_aa_chunk_coord =
        observer_chunk_coord - TerrainChunkCoord::new(load_radius, 0, load_radius);
    let load_bb_chunk_coord =
        observer_chunk_coord + TerrainChunkCoord::new(load_radius, 0, load_radius);

    // 计算需要加载的 chunks（不考虑 margin）
    let mut chunks_to_load: HashSet<TerrainChunkCoord> = HashSet::new();
    for x in load_aa_chunk_coord.x()..=load_bb_chunk_coord.x() {
        for z in load_aa_chunk_coord.z()..=load_bb_chunk_coord.z() {
            for dy in height_range.clone() {
                let chunk_y = observer_chunk_coord.y() + dy;
                let coord = TerrainChunkCoord::new(x, chunk_y, z);
                chunks_to_load.insert(coord);
            }
        }
    }

    trace!(
        "Desired Chunks Count: {:?} -> {:?}",
        chunks_to_load.len(),
        chunks_to_load
    );

    // 卸载范围：使用 visibility_radius + margin，提供缓冲区
    let unload_radius = (visibility_radius + margin) as i32;
    let unload_aa_chunk_coord =
        observer_chunk_coord - TerrainChunkCoord::new(unload_radius, 0, unload_radius);
    let unload_bb_chunk_coord =
        observer_chunk_coord + TerrainChunkCoord::new(unload_radius, 0, unload_radius);

    // 找出需要卸载的 chunks（超出卸载范围的才卸载）
    let mut chunks_to_unload = Vec::new();
    for (coord, _entity) in loaded_chunks.iter() {
        // 检查是否超出卸载范围
        if (coord.x() < unload_aa_chunk_coord.x() || coord.x() > unload_bb_chunk_coord.x())
            || (coord.z() < unload_aa_chunk_coord.z() || coord.z() > unload_bb_chunk_coord.z())
            || (coord.y() < observer_chunk_coord.y() + height_range.start()
                || coord.y() > observer_chunk_coord.y() + height_range.end())
        {
            chunks_to_unload.push(*coord);
        }
    }

    trace!(
        "Chunks to Unload Count: {:?} -> {:?}",
        chunks_to_unload.len(),
        chunks_to_unload
    );

    // 找出需要加载的 chunks
    let mut to_load_chunks = Vec::new();
    for coord in chunks_to_load {
        if !loaded_chunks.contains(&coord) {
            to_load_chunks.push(coord);
        }
    }

    trace!(
        "Chunks to Load Count: {:?} -> {:?}",
        to_load_chunks.len(),
        to_load_chunks
    );

    // 卸载 chunks
    for coord in chunks_to_unload {
        if let Some(entity) = loaded_chunks.remove(&coord) {
            commands
                .entity(entity)
                .despawn_related::<TerrainChunkVisual>();
            commands.entity(entity).despawn();
            unload_requests.write(TerrainChunkUnloadMsg { coord });
        }
    }

    // 加载 chunks
    for coord in to_load_chunks {
        let entity = commands
            .spawn((
                TerrainChunk,
                coord,
                // LOD 已移除 — 所有 chunk 固定最大精度
                Name::new(format!("Chunk_{:?}", coord)),
            ))
            .id();
        loaded_chunks.insert(coord, entity);
        load_requests.write(TerrainChunkLoadMsg { coord });
    }
}
