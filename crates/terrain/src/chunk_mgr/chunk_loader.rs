use std::collections::BinaryHeap;

// FIXME 有时候没有删除chunk内容。
use crate::{
    lod::{lod_octree::TerrainLodOctree, morton_code::MortonCode},
    TerrainObserver,
};
use bevy::{
    math::{
        bounding::{Aabb3d, BoundingVolume, IntersectsVolume},
        Affine3A,
    },
    prelude::*,
    render::{
        camera::CameraProjection,
        primitives::{Aabb, Frustum},
    },
    utils::HashSet,
};
use smallvec::SmallVec;

use super::TerrainChunkSystemSet;

// 1. 创建所有的主mesh，这样填充缝隙的时候才能正确处理。之后处理缝隙
// 2. 处理主mesh的过程中有了新的主mesh，正常继续推进。
// 3. 如果处理缝隙的过程中有了新的主mesh，需要暂停处理缝隙。重新开始处理主mesh。
// 4. 这种必须最后再删除，避免闪烁。在性能不足时，可能缓存过多的chunk，导致内存占用过高。
// 不这样做的话，最好是单帧处理完毕，就没有缓存队列的问题了。
#[derive(Debug, Default)]
pub struct TerrainChunkLoaderPlugin;

impl Plugin for TerrainChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TerrainChunkLoadEvent>()
            .add_event::<TerrainChunkUnLoadEvent>()
            .init_resource::<TerrainChunkLoader>()
            .add_systems(
                Update,
                (
                    update_loader_state,
                    to_load_chunk,
                    to_unload_chunk,
                    reload_terrain_chunk,
                )
                    .chain()
                    .in_set(TerrainChunkSystemSet::UpdateLoader),
            );
    }
}

#[derive(Debug, PartialEq, Default, Eq, Clone, Copy)]
pub struct LeafNodeKey {
    pub address: MortonCode,

    pub is_in_frustums: bool,
    pub distance_squared: u64,
    // unit is degree
    pub angle: u32,
}

impl Ord for LeafNodeKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[allow(clippy::non_canonical_partial_ord_impl)]
impl PartialOrd for LeafNodeKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.is_in_frustums.partial_cmp(&other.is_in_frustums) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match self.distance_squared.partial_cmp(&other.distance_squared) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord.map(|ord| ord.reverse()),
        }
        self.angle
            .partial_cmp(&other.angle)
            .map(|ord| ord.reverse())
    }
}

/// 需要保证最后删除，避免闪烁。
/// 同时，如果有需要加载的，也应该加载。
#[derive(Resource, Debug, Default)]
pub struct TerrainChunkLoader {
    pub leaf_node_pending_load_deque: BinaryHeap<LeafNodeKey>,
    pub pending_unload_leaf_node_set: HashSet<MortonCode>,
    pub pending_reload_aabb_vec: Vec<Aabb3d>,
}

impl TerrainChunkLoader {
    pub fn add_reload_aabb(&mut self, aabb: Aabb3d) {
        self.pending_reload_aabb_vec.push(aabb);
    }
}

type FrustumsType = SmallVec<[Frustum; 1]>;
type GlobalTransformsType = SmallVec<[GlobalTransform; 1]>;

fn update_leaf_node_data(
    leaf_node_key: &mut LeafNodeKey,
    frustums: &FrustumsType,
    global_transforms: &GlobalTransformsType,
    node_aabb: &Aabb,
) {
    let mut is_in_frustums = false;
    for frustum in frustums.iter() {
        if frustum.intersects_obb(node_aabb, &Affine3A::IDENTITY, true, true) {
            is_in_frustums = true;
            break;
        }
    }
    let mut min_distance_squared = u64::MAX;
    let mut min_angle = u32::MAX;
    for global_transform in global_transforms.iter() {
        let (_, rotation, translation) = global_transform.to_scale_rotation_translation();
        let leaf_node_location: Vec3 = node_aabb.center.into();
        let relative_translation = leaf_node_location - translation;
        min_distance_squared =
            min_distance_squared.min(relative_translation.length_squared() as u64);

        let (axis, _angle) = rotation.to_axis_angle();
        min_angle = min_angle.min(relative_translation.angle_between(axis).to_degrees() as u32);
    }

    leaf_node_key.is_in_frustums = is_in_frustums;
    leaf_node_key.distance_squared = min_distance_squared;
    leaf_node_key.angle = min_angle;
}

#[allow(clippy::type_complexity)]
pub fn update_loader_state(
    mut loader: ResMut<TerrainChunkLoader>,
    observer_query: Query<(&GlobalTransform, &Projection), With<TerrainObserver>>,
    lod_octree: Res<TerrainLodOctree>,
) {
    let mut frustums = FrustumsType::new();
    let mut global_transforms = GlobalTransformsType::new();
    for (global_transform, projection) in observer_query.iter() {
        let frustum = projection.compute_frustum(global_transform);
        frustums.push(frustum);
        global_transforms.push(*global_transform);
    }

    loader.leaf_node_pending_load_deque.clear();

    for level in lod_octree.octree_levels.iter() {
        // to add nodes
        for node in level.get_added_nodes() {
            let mut leaf_node_key = LeafNodeKey {
                address: node.code,
                ..Default::default()
            };
            let node_aabb = Aabb {
                center: node.aabb.center(),
                half_extents: node.aabb.half_size(),
            };
            update_leaf_node_data(
                &mut leaf_node_key,
                &frustums,
                &global_transforms,
                &node_aabb,
            );
            // if leaf_node_key.distance_squared < 1000000 {
            // loader.leaf_node_pending_load_deque.push(leaf_node_key);
            // }
            loader.leaf_node_pending_load_deque.push(leaf_node_key);
        }

        // to remove nodes
        for node in level.get_removed_nodes() {
            loader.pending_unload_leaf_node_set.insert(node.code);
        }
    }

    debug!(
        "loader.leaf_node_pending_load_deque :{}, loader.pending_unload_leaf_node_set: {}",
        loader.leaf_node_pending_load_deque.len(),
        loader.pending_unload_leaf_node_set.len()
    );
}

pub fn to_load_chunk(mut loader: ResMut<TerrainChunkLoader>, mut commands: Commands) {
    let num = loader.leaf_node_pending_load_deque.len();
    if num == 0 {
        return;
    }

    let mut load_event = TerrainChunkLoadEvent {
        node_addresses: Vec::with_capacity(num),
    };

    while let Some(key) = loader.leaf_node_pending_load_deque.pop() {
        debug!("to load lod octree node: {:?}", key.address);
        load_event.node_addresses.push(key.address);
    }

    debug!("to load chunk: {:?}", load_event.node_addresses.len());
    // 防止显存占用过多，卡死。
    if load_event.node_addresses.len() < 1500 {
        commands.trigger(load_event);
    }
}

pub fn to_unload_chunk(mut loader: ResMut<TerrainChunkLoader>, mut commands: Commands) {
    if loader.pending_unload_leaf_node_set.is_empty() {
        return;
    }

    let unload_nodes = std::mem::take(&mut loader.pending_unload_leaf_node_set);
    let unload_event = TerrainChunkUnLoadEvent {
        node_addresses: unload_nodes.into_iter().collect(),
    };
    debug!("to unload chunk: {:?}", unload_event.node_addresses.len());
    commands.trigger(unload_event);
}

#[derive(Event, Debug)]
pub struct TerrainChunkLoadEvent {
    pub node_addresses: Vec<MortonCode>,
}

#[derive(Event, Debug)]
pub struct TerrainChunkUnLoadEvent {
    pub node_addresses: Vec<MortonCode>,
}

#[derive(Event, Debug)]
pub struct TerrainChunkReloadEvent {
    pub node_addresses: Vec<MortonCode>,
}

pub fn reload_terrain_chunk(
    mut loader: ResMut<TerrainChunkLoader>,
    lod_octree: Res<TerrainLodOctree>,
    mut commands: Commands,
) {
    if loader.pending_reload_aabb_vec.is_empty() {
        return;
    }

    let mut intersects_nodes = vec![];
    for level in lod_octree.octree_levels.iter() {
        for node in level.get_current() {
            for aabb in loader.pending_reload_aabb_vec.iter() {
                if node.aabb.intersects(aabb) {
                    intersects_nodes.push(node.code);
                }
            }
        }
    }
    loader.pending_reload_aabb_vec.clear();

    commands.trigger(TerrainChunkReloadEvent {
        node_addresses: intersects_nodes,
    });
}
