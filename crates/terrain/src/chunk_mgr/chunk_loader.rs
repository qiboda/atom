use std::{
    hash::{Hash, Hasher},
    ops::Not,
};

use crate::{
    lod::{
        lod_octree::{
            ObserverFrustums, ObserverGlobalTransforms, TerrainLodOctree, TerrainLodOctreeNode,
        },
        morton_code::MortonCode,
    },
    setting::TerrainSetting,
    TerrainObserver,
};
use bevy::{
    math::{bounding::BoundingVolume, Affine3A},
    prelude::*,
    render::{
        camera::CameraProjection,
        primitives::{Aabb, Sphere},
    },
    utils::HashMap,
};

use super::TerrainChunkSystemSet;

// 1. 只加载视锥体内的chunk，离开视锥并不删除。
// 2. 每帧增加和删除的chunk进行队列缓存。
// 3. 之后根据条件进行加载和删除。
// 4. 删除前进行检测，如果在视锥体内，查看是否父类或者所有子类是否都已经加载了。
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
                    update_loaded_leaf_node_info,
                    update_loader_state,
                    to_load_chunk,
                    to_unload_chunk,
                    to_reload_chunk,
                )
                    .chain()
                    .in_set(TerrainChunkSystemSet::UpdateLoader),
            );
    }
}

#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub struct LeafNodeKey {
    pub address: MortonCode,
    pub aabb: Aabb,

    pub is_in_frustums: bool,
    pub distance_squared: u64,
    // unit is degree
    pub angle: u32,
}

impl LeafNodeKey {
    pub fn from_lod_leaf_node(node: &TerrainLodOctreeNode) -> Self {
        Self {
            address: node.code,
            aabb: Aabb {
                center: node.aabb.center(),
                half_extents: node.aabb.half_size(),
            },
            ..Default::default()
        }
    }
}

impl Hash for LeafNodeKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
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

#[derive(Debug, Default)]
pub struct LoadedNodeInfo {
    pub loaded_frame_count: usize,
}

/// 需要保证最后删除，避免闪烁。
/// 同时，如果有需要加载的，也应该加载。
#[derive(Resource, Debug, Default)]
pub struct TerrainChunkLoader {
    pub pending_load_leaf_node_map: HashMap<MortonCode, LeafNodeKey>,
    pub pending_unload_leaf_node_map: HashMap<MortonCode, LeafNodeKey>,

    pub loaded_leaf_node_map: HashMap<MortonCode, LoadedNodeInfo>,

    pub pending_reload_leaf_node_map: HashMap<MortonCode, LeafNodeKey>,
}

impl TerrainChunkLoader {
    /// TODO reload 是否需要加载缝隙。以及加载哪些缝隙。
    /// reload 的分类，csg操作或者其他操作。
    pub fn insert_pending_reload_leaf_node_map(
        &mut self,
        morton_code: MortonCode,
        leaf_node_key: LeafNodeKey,
    ) {
        self.pending_reload_leaf_node_map
            .insert(morton_code, leaf_node_key);
    }

    pub fn is_loaded(&self, morton_code: &MortonCode) -> bool {
        self.loaded_leaf_node_map.contains_key(morton_code)
    }

    pub fn can_reload(&self, morton_code: &MortonCode) -> bool {
        self.loaded_leaf_node_map.contains_key(morton_code)
    }

    pub fn can_unload(&self, morton_code: &MortonCode) -> bool {
        if let Some(loaded_info) = self.loaded_leaf_node_map.get(morton_code) {
            // 因为节点和子节点或者父节点是同时添加和删除的，以此可以通过帧数来判断是否可以删除。
            // 添加的子节点或者父节点过两帧后会创建出来，所以这里判断是否大于2帧。
            if loaded_info.loaded_frame_count > 2 {
                return true;
            }
        }
        false
    }
}

fn update_loaded_leaf_node_info(mut loader: ResMut<TerrainChunkLoader>) {
    loader
        .loaded_leaf_node_map
        .iter_mut()
        .for_each(|(_, info)| {
            info.loaded_frame_count += 1;
        });
}

fn update_leaf_node_data(
    leaf_node_key: &mut LeafNodeKey,
    frustums: &ObserverFrustums,
    terrain_setting: &Res<TerrainSetting>,
) {
    let mut is_in_frustums = false;
    for frustum in frustums.iter() {
        let sphere = Sphere {
            center: leaf_node_key.aabb.center,
            radius: leaf_node_key.aabb.half_extents.length(),
        };
        // Do quick sphere-based frustum culling
        if frustum.intersects_sphere(&sphere, terrain_setting.camera_far_limit) {
            // Do more precise OBB-based frustum culling
            if frustum.intersects_obb(
                &leaf_node_key.aabb,
                &Affine3A::IDENTITY,
                true,
                terrain_setting.camera_far_limit,
            ) {
                is_in_frustums = true;
                break;
            }
        }
    }
    // let mut min_distance_squared = u64::MAX;
    // let mut min_angle = u32::MAX;
    // for global_transform in global_transforms.iter() {
    //     let (_, rotation, translation) = global_transform.to_scale_rotation_translation();
    //     let leaf_node_location: Vec3 = leaf_node_key.aabb.center.into();
    //     let relative_translation = leaf_node_location - translation;
    //     min_distance_squared =
    //         min_distance_squared.min(relative_translation.length_squared() as u64);

    //     let (axis, _angle) = rotation.to_axis_angle();
    //     min_angle = min_angle.min(relative_translation.angle_between(axis).to_degrees() as u32);
    // }

    leaf_node_key.is_in_frustums = is_in_frustums;
    // leaf_node_key.distance_squared = min_distance_squared;
    // leaf_node_key.angle = min_angle;
}

#[allow(clippy::type_complexity)]
pub fn update_loader_state(
    mut loader: ResMut<TerrainChunkLoader>,
    observer_query: Query<(&GlobalTransform, &Projection), With<TerrainObserver>>,
    lod_octree: Res<TerrainLodOctree>,
    terrain_setting: Res<TerrainSetting>,
) {
    let mut frustums = ObserverFrustums::new();
    let mut global_transforms = ObserverGlobalTransforms::new();
    for (global_transform, projection) in observer_query.iter() {
        let frustum = projection.compute_frustum(global_transform);
        frustums.push(frustum);
        global_transforms.push(*global_transform);
    }

    for level in lod_octree.octree_levels.iter() {
        // to add nodes
        for node in level.get_added_nodes() {
            let leaf_node_key = LeafNodeKey::from_lod_leaf_node(node);
            loader.pending_unload_leaf_node_map.remove(&node.code);
            loader
                .pending_load_leaf_node_map
                .insert(node.code, leaf_node_key);
        }

        // to remove nodes
        for node in level.get_removed_nodes() {
            let leaf_node_key = LeafNodeKey::from_lod_leaf_node(node);
            loader.pending_load_leaf_node_map.remove(&node.code);
            loader
                .pending_unload_leaf_node_map
                .insert(node.code, leaf_node_key);
        }
    }

    for (_code, leaf_node_key) in loader.pending_load_leaf_node_map.iter_mut() {
        update_leaf_node_data(leaf_node_key, &frustums, &terrain_setting);
    }

    for (_code, leaf_node_key) in loader.pending_unload_leaf_node_map.iter_mut() {
        update_leaf_node_data(leaf_node_key, &frustums, &terrain_setting);
    }

    for (_code, leaf_node_key) in loader.pending_reload_leaf_node_map.iter_mut() {
        update_leaf_node_data(leaf_node_key, &frustums, &terrain_setting);
    }

    debug!(
        "loader.leaf_node_pending_load_deque :{}, loader.pending_unload_leaf_node_set: {}, loader.pending_reload_leaf_node_set: {}",
        loader.pending_load_leaf_node_map.len(),
        loader.pending_unload_leaf_node_map.len(),
        loader.pending_reload_leaf_node_map.len()
    );
}

pub fn to_load_chunk(
    mut loader: ResMut<TerrainChunkLoader>,
    mut commands: Commands,
    mut last_num: Local<usize>,
) {
    let mut load_event = TerrainChunkLoadEvent {
        node_addresses: Vec::with_capacity(*last_num),
    };

    let to_load_nodes = loader
        .pending_load_leaf_node_map
        .iter()
        .filter(|(_, key)| loader.is_loaded(&key.address).not() && key.is_in_frustums);

    to_load_nodes.for_each(|(code, _key)| {
        load_event.node_addresses.push(*code);
    });

    if load_event.node_addresses.is_empty() {
        return;
    }

    for address in load_event.node_addresses.iter() {
        loader.pending_load_leaf_node_map.remove(address);
        loader
            .loaded_leaf_node_map
            .insert(*address, LoadedNodeInfo::default());
    }

    *last_num = load_event.node_addresses.len();

    debug!("to load chunk: {:?}", load_event.node_addresses.len());
    // 防止显存占用过多，卡死。
    if load_event.node_addresses.len() < 1500 {
        commands.trigger(load_event);
    }
}

pub fn to_unload_chunk(
    mut loader: ResMut<TerrainChunkLoader>,
    mut commands: Commands,
    mut last_num: Local<usize>,
) {
    let mut load_event = TerrainChunkUnLoadEvent {
        node_addresses: Vec::with_capacity(*last_num),
    };

    let to_unload_nodes = loader
        .pending_unload_leaf_node_map
        .iter()
        .filter(|(_, key)| {
            (loader.can_unload(&key.address) && key.is_in_frustums) || key.is_in_frustums.not()
        });

    to_unload_nodes.for_each(|(code, _key)| {
        load_event.node_addresses.push(*code);
    });

    if load_event.node_addresses.is_empty() {
        return;
    }

    for address in load_event.node_addresses.iter() {
        loader.pending_unload_leaf_node_map.remove(address);
        loader.loaded_leaf_node_map.remove(address);
    }

    *last_num = load_event.node_addresses.len();

    debug!("to unload chunk: {:?}", load_event.node_addresses.len());
    commands.trigger(load_event);
}

pub fn to_reload_chunk(mut loader: ResMut<TerrainChunkLoader>, mut commands: Commands) {
    if loader.pending_reload_leaf_node_map.is_empty() {
        return;
    }

    let mut reload_event = TerrainChunkReloadEvent {
        node_addresses: Vec::with_capacity(loader.pending_reload_leaf_node_map.len()),
    };

    let to_reload_nodes = loader
        .pending_reload_leaf_node_map
        .iter()
        .filter(|(_, key)| loader.can_reload(&key.address) && key.is_in_frustums);

    to_reload_nodes.for_each(|(code, _key)| {
        reload_event.node_addresses.push(*code);
    });

    for code in reload_event.node_addresses.iter() {
        loader.pending_reload_leaf_node_map.remove(code);
    }

    debug!("to reload chunk: {:?}", reload_event.node_addresses);

    commands.trigger(reload_event);
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

#[cfg(test)]
mod tests {
    use bevy::{
        math::{Affine3A, Vec3},
        prelude::{GlobalTransform, Projection},
        render::{camera::CameraProjection, primitives::Aabb},
    };

    #[test]
    fn test_frustum_intersect_with_obb() {
        let transform = GlobalTransform::default();
        let projection = Projection::default();
        let frustum = projection.compute_frustum(&transform);

        let aabb = Aabb::from_min_max(Vec3::splat(100.0), Vec3::splat(101.0));
        assert!(!frustum.intersects_obb(&aabb, &Affine3A::IDENTITY, true, true));

        let aabb = Aabb::from_min_max(Vec3::splat(-1.0), Vec3::splat(1.0));
        assert!(frustum.intersects_obb(&aabb, &Affine3A::IDENTITY, true, true));

        let aabb = Aabb::from_min_max(Vec3::new(100.0, 0.0, 0.0), Vec3::new(101.0, 1.0, 1.0));
        assert!(frustum.intersects_obb(&aabb, &Affine3A::IDENTITY, true, true));
    }
}
