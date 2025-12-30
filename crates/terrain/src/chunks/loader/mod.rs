pub mod observer;

use std::ops::{Not, RangeInclusive};

use bevy::{
    platform::collections::{HashMap, HashSet},
    prelude::*,
};

use crate::{
    chunks::{
        chunk::{TerrainChunk, TerrainChunkCoord, TerrainChunkLod},
        loader::observer::{TerrainObserver, TerrainObserverConfig},
    },
    terrain::{TerrainSystems, setting::TerrainSetting},
};

/// Chunk 加载请求消息
#[derive(Message, Debug, Clone)]
pub struct TerrainChunkLoadMsg {
    pub lod: u8,
    // lod0 coord
    pub coord: TerrainChunkCoord,
}

/// Chunk 卸载请求消息
#[derive(Message, Debug, Clone)]
pub struct TerrainChunkUnloadMsg {
    pub lod: u8,
    // lod0 coord
    pub coord: TerrainChunkCoord,
}

#[derive(Default, Debug)]
pub struct TerrainChunks {
    // coord 是 对应 lod 的 chunk 坐标 ，并非总是lod0的coord。
    chunks: HashMap<TerrainChunkCoord, Entity>,
}

impl TerrainChunks {
    pub fn insert(&mut self, lod_coord: TerrainChunkCoord, entity: Entity) {
        self.chunks.insert(lod_coord, entity);
    }

    pub fn remove(&mut self, lod_coord: &TerrainChunkCoord) -> Option<Entity> {
        self.chunks.remove(lod_coord)
    }

    pub fn contains(&self, lod_coord: &TerrainChunkCoord) -> bool {
        self.chunks.contains_key(lod_coord)
    }

    pub fn get(&self, lod_coord: &TerrainChunkCoord) -> Option<Entity> {
        self.chunks.get(lod_coord).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&TerrainChunkCoord, &Entity)> {
        self.chunks.iter()
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

/// 追踪已加载的 chunks
#[derive(Resource, Default, Debug)]
pub struct TerrainLoadedChunks {
    // lod -> loaded chunks
    lod_chunks: HashMap<u8, TerrainChunks>,
}

impl TerrainLoadedChunks {
    /**
     * 插入已加载的 chunk。其中`coord`是 lod0的Chunk坐标
     */
    pub fn insert(&mut self, lod: u8, coord: TerrainChunkCoord, entity: Entity) {
        let lod_coord = coord.lod_bias_up(lod); // 获取对应的 LOD 级别
        self.lod_chunks
            .entry(lod)
            .or_default()
            .insert(lod_coord, entity);
    }

    /**
     * `coord`是 lod0的Chunk坐标
     */
    pub fn remove(&mut self, lod: u8, coord: &TerrainChunkCoord) -> Option<Entity> {
        let lod_coord = coord.lod_bias_up(lod); // 获取对应的 LOD 级别
        if let Some(lod_chunks) = self.lod_chunks.get_mut(&lod) {
            return lod_chunks.remove(&lod_coord);
        }
        None
    }

    /**
     * `coord`是 lod0的Chunk坐标
     */
    pub fn contains(&self, lod: u8, coord: &TerrainChunkCoord) -> bool {
        let lod_coord = coord.lod_bias_up(lod); // 获取对应的 LOD 级别
        if let Some(lod_chunks) = self.lod_chunks.get(&lod) {
            return lod_chunks.contains(&lod_coord);
        }
        false
    }

    /**
     * `coord` is lod0 coord
     */
    pub fn get(&self, lod: u8, coord: &TerrainChunkCoord) -> Option<Entity> {
        let lod_coord = coord.lod_bias_up(lod); // 获取对应的 LOD 级别
        if let Some(lod_chunks) = self.lod_chunks.get(&lod) {
            return lod_chunks.get(&lod_coord);
        }
        None
    }

    fn get_chunks_by_lod(&mut self, lod: u8) -> Option<&mut TerrainChunks> {
        if let Some(lod_chunks) = self.lod_chunks.get_mut(&lod) {
            return Some(lod_chunks);
        }
        None
    }

    pub fn lods(&self) -> impl Iterator<Item = u8> {
        self.lod_chunks.keys().copied()
    }

    pub fn iter_mut(
        &mut self,
    ) -> bevy::platform::collections::hash_map::IterMut<'_, u8, TerrainChunks> {
        self.lod_chunks.iter_mut()
    }

    pub fn iter(&self) -> bevy::platform::collections::hash_map::Iter<'_, u8, TerrainChunks> {
        self.lod_chunks.iter()
    }
}

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

/// Clipmap chunk loader 主系统
pub fn update_clipmap_chunks(
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
    let visibility_radius = observer_config.terrain_load_radius.unwrap_or(1);

    let height_range = get_terrain_height_range(&terrain_setting, observer_config);

    // 计算观察者所在的 chunk 坐标
    let observer_pos = observer_transform.translation();

    let observer_chunk_coord = TerrainChunkCoord::from_world_pos(observer_pos, chunk_size);
    let aa_chunk_coord = observer_chunk_coord
        - TerrainChunkCoord::new(visibility_radius as i32, 0, visibility_radius as i32);
    let bb_chunk_coord = observer_chunk_coord
        + TerrainChunkCoord::new(visibility_radius as i32, 0, visibility_radius as i32);

    // coord is lod coord
    let mut desired_chunks: HashMap<u8, HashSet<TerrainChunkCoord>> = HashMap::new();

    // 计算需要加载的 chunks，
    // 根据距离计算lod，再根据高度和视锥体裁剪掉多余的chunk。
    // TODO 视锥体裁剪 不需要了，仅仅用于确定顺序。
    for lod in 0..terrain_setting.clipmap_config.lod_count {
        let last_lod_chunk_coords = if lod > 0 {
            desired_chunks.get(&(lod - 1)).cloned().unwrap_or_default()
        } else {
            HashSet::new()
        };
        let lod_chunk_coords = desired_chunks.entry(lod).or_default();

        for x in aa_chunk_coord.x()..=bb_chunk_coord.x() {
            for z in aa_chunk_coord.z()..=bb_chunk_coord.z() {
                let cur_coord = TerrainChunkCoord::new(x, observer_chunk_coord.y(), z);
                let xz_distance = observer_chunk_coord.chebyshev_distance_xz(&cur_coord);

                let lod_radius = terrain_setting.get_clipmap_radius_by_lod(lod);
                let last_lod_radius = if lod > 0 {
                    terrain_setting.get_clipmap_radius_by_lod(lod - 1)
                } else {
                    0
                };

                // 观察者在这个 LOD 范围内，继续处理
                if (last_lod_radius as i32) <= xz_distance && xz_distance < (lod_radius as i32) {
                    for dy in height_range.clone() {
                        let chunk_y = observer_chunk_coord.y() + dy;
                        let coord_with_height =
                            TerrainChunkCoord::new(cur_coord.x(), chunk_y, cur_coord.z());
                        if lod > 0 {
                            // 如果上一个 LOD 已经包含这个 chunk，则跳过
                            if last_lod_chunk_coords
                                .contains(&coord_with_height.lod_bias_up(lod - 1))
                            {
                                continue;
                            }
                        }

                        let lod_coord_with_height = coord_with_height.lod_bias_up(lod);

                        // let aabb = Aabb::from_min_max(
                        //     lod_coord_with_height.to_world_pos(lod_chunk_size),
                        //     (lod_coord_with_height + TerrainChunkCoord::new(1, 1, 1))
                        //         .to_world_pos(lod_chunk_size),
                        // );

                        // if observer_frustum
                        //     .intersects_obb(&aabb, &Affine3A::IDENTITY, true, false)
                        //     .not()
                        // {
                        //     continue;
                        // }

                        lod_chunk_coords.insert(lod_coord_with_height);
                    }
                } else {
                    // 观察者超出这个 LOD 范围，跳过更高 LOD
                    continue;
                }
            }
        }
    }

    for (lod, coords) in desired_chunks.iter() {
        trace!("Desired Chunks LOD {} Count: {:?}", lod, coords.len(),);
    }

    // 找出需要卸载的 chunks
    for (&lod, loaded_lod_chunks) in loaded_chunks.iter_mut() {
        let mut lod_chunks_to_unload = Vec::new();
        if let Some(chunk_coords) = desired_chunks.get(&lod) {
            for (lod_coord, _entity) in loaded_lod_chunks.iter() {
                if chunk_coords.contains(lod_coord).not() {
                    lod_chunks_to_unload.push(*lod_coord);
                }
            }
        } else {
            for (lod_coord, _entity) in loaded_lod_chunks.iter() {
                lod_chunks_to_unload.push(*lod_coord);
            }
        }

        trace!(
            "Chunks to Unload Count: {:?} -> {:?}",
            lod_chunks_to_unload.len(),
            lod_chunks_to_unload
        );

        // 卸载 chunks
        for lod_coord in lod_chunks_to_unload {
            if let Some(entity) = loaded_lod_chunks.remove(&lod_coord) {
                commands.entity(entity).despawn();
                let coord = lod_coord.lod_bias_down(lod);
                unload_requests.write(TerrainChunkUnloadMsg { lod, coord });
            }
        }
    }

    for (lod, lod_coords) in desired_chunks {
        let mut lod_chunks_to_load = HashSet::new();
        if let Some(loaded_lod_chunks) = loaded_chunks.get_chunks_by_lod(lod) {
            for lod_coord in lod_coords {
                if !loaded_lod_chunks.contains(&lod_coord) {
                    lod_chunks_to_load.insert(lod_coord);
                }
            }
        } else {
            for lod_coord in lod_coords {
                lod_chunks_to_load.insert(lod_coord);
            }
        }

        trace!(
            "Chunks to Load LOD {} Count: {:?}",
            lod,
            lod_chunks_to_load.len(),
        );

        // 找出需要加载的 chunks
        for lod_coord in lod_chunks_to_load {
            load_requests.write(TerrainChunkLoadMsg {
                lod,
                coord: lod_coord.lod_bias_down(lod),
            });
        }
    }
}

/// TODO: 是否合并到 update_clipmap_chunks 中？
/// 处理 chunk 加载请求（创建空实体，实际生成由其他系统处理）
pub fn handle_chunk_load_requests(
    mut commands: Commands,
    mut load_requests: MessageReader<TerrainChunkLoadMsg>,
    mut loaded_chunks: ResMut<TerrainLoadedChunks>,
) {
    for request in load_requests.read() {
        if let Some(loaded_lod_chunks) = loaded_chunks.get_chunks_by_lod(request.lod) {
            if loaded_lod_chunks.contains(&request.coord.lod_bias_up(request.lod)) {
                continue;
            }
        }

        let entity = commands
            .spawn((
                TerrainChunk,
                request.coord,
                TerrainChunkLod { lod: request.lod },
                Name::new(format!("Chunk_{:?}_LOD{}", request.coord, request.lod)),
            ))
            .id();

        loaded_chunks.insert(request.lod, request.coord, entity);
        info!("加载 Chunk: {:?}, LOD: {}", request.coord, request.lod);
    }
}

/// TODO: 是否合并到 update_clipmap_chunks 中？
pub fn handle_chunk_unload_requests(
    mut commands: Commands,
    mut unload_requests: MessageReader<TerrainChunkUnloadMsg>,
    mut loaded_chunks: ResMut<TerrainLoadedChunks>,
) {
    for request in unload_requests.read() {
        if let Some(entity) = loaded_chunks.remove(request.lod, &request.coord) {
            commands.entity(entity).despawn();
            info!("卸载 Chunk: {:?}, LOD: {}", request.coord, request.lod);
        }
    }
}

/// Chunk loader 系统集
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum ChunkLoaderSystems {
    /// 更新 clipmap
    UpdateClipmap,
    /// 处理加载请求
    HandleLoadRequests,
    /// 处理卸载请求
    HandleUnloadRequests,
}

/// 添加 chunk loader 插件辅助函数
pub fn add_chunk_loader_systems(app: &mut App) {
    app.init_resource::<TerrainLoadedChunks>()
        .add_message::<TerrainChunkLoadMsg>()
        .add_message::<TerrainChunkUnloadMsg>()
        .configure_sets(
            Update,
            (
                ChunkLoaderSystems::UpdateClipmap,
                ChunkLoaderSystems::HandleLoadRequests,
                ChunkLoaderSystems::HandleUnloadRequests,
            )
                .in_set(TerrainSystems::ChunkLoader)
                .chain(),
        )
        .add_systems(
            Update,
            (
                update_clipmap_chunks.in_set(ChunkLoaderSystems::UpdateClipmap),
                handle_chunk_load_requests.in_set(ChunkLoaderSystems::HandleLoadRequests),
                handle_chunk_unload_requests.in_set(ChunkLoaderSystems::HandleUnloadRequests),
            ),
        );
}
