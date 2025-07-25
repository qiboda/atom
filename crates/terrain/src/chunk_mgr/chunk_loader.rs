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
    math::bounding::{Aabb3d, BoundingVolume, IntersectsVolume},
    prelude::*,
    render::{camera::CameraProjection, primitives::Aabb},
    utils::HashMap,
};

use super::TerrainChunkSystemSet;

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

    pub is_in_base_range: bool,
    pub is_in_frustums: bool,
    pub is_in_height: bool,
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

    pub fn can_load(&self) -> bool {
        (self.is_in_frustums || self.is_in_base_range) && self.is_in_height
    }
}

impl Hash for LeafNodeKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.address.hash(state);
    }
}

#[derive(Debug)]
pub struct LoadedNodeInfo {
    pub to_unload_wait_frame_count: usize,
    pub leaf_node_key: LeafNodeKey,
}

impl LoadedNodeInfo {
    pub fn new(leaf_node_key: LeafNodeKey) -> Self {
        Self {
            leaf_node_key,
            to_unload_wait_frame_count: 0,
        }
    }
}

/// 需要保证最后删除，避免闪烁。
/// 同时，如果有需要加载的，也应该加载。
#[derive(Resource, Debug, Default)]
pub struct TerrainChunkLoader {
    pub loaded_leaf_node_map: HashMap<MortonCode, LoadedNodeInfo>,

    pub pending_load_leaf_node_map: HashMap<MortonCode, LeafNodeKey>,
    pub pending_unload_leaf_node_map: HashMap<MortonCode, LeafNodeKey>,

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
            if loaded_info.to_unload_wait_frame_count > 3 {
                return true;
            }
        }
        false
    }
}

fn update_leaf_node_data(
    leaf_node_key: &mut LeafNodeKey,
    _frustums: &ObserverFrustums,
    global_transforms: &ObserverGlobalTransforms,
    terrain_setting: &Res<TerrainSetting>,
) {
    let mut is_in_height = false;
    if leaf_node_key.aabb.max().y >= *terrain_setting.height_visibility_range.start()
        && leaf_node_key.aabb.min().y <= *terrain_setting.height_visibility_range.end()
    {
        is_in_height = true;
    }

    // let mut is_in_frustums = false;
    // for frustum in frustums.iter() {
    //     let sphere = Sphere {
    //         center: leaf_node_key.aabb.center,
    //         radius: leaf_node_key.aabb.half_extents.length(),
    //     };
    //     // Do quick sphere-based frustum culling
    //     if frustum.intersects_sphere(&sphere, terrain_setting.camera_far_limit) {
    //         // Do more precise OBB-based frustum culling
    //         if frustum.intersects_obb(
    //             &leaf_node_key.aabb,
    //             &Affine3A::IDENTITY,
    //             true,
    //             terrain_setting.camera_far_limit,
    //         ) {
    //             is_in_frustums = true;
    //             break;
    //         }
    //     }
    // }

    let mut is_in_base_range = false;
    for global_transform in global_transforms.iter() {
        let translation = global_transform.translation();
        let observer_aabb = Aabb3d::new(
            translation,
            Vec3::splat(terrain_setting.horizontal_visibility_range),
        );
        let aabb = Aabb3d::new(leaf_node_key.aabb.center, leaf_node_key.aabb.half_extents);
        if observer_aabb.intersects(&aabb) {
            is_in_base_range = true;
        }
    }

    leaf_node_key.is_in_frustums = false;
    leaf_node_key.is_in_height = is_in_height;
    leaf_node_key.is_in_base_range = is_in_base_range;
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

    loader.pending_load_leaf_node_map.clear();
    loader.pending_unload_leaf_node_map.clear();

    let mut to_load_nodes = HashMap::new();
    for level in lod_octree.octree_levels.iter() {
        for node in level.get_current().iter() {
            let mut leaf_node_key = LeafNodeKey::from_lod_leaf_node(node);
            update_leaf_node_data(
                &mut leaf_node_key,
                &frustums,
                &global_transforms,
                &terrain_setting,
            );

            if leaf_node_key.can_load() {
                to_load_nodes.insert(node.code, leaf_node_key);
            }
        }
    }

    let mut to_unload_nodes = HashMap::new();
    for (code, loaded_node) in loader.loaded_leaf_node_map.iter_mut() {
        if to_load_nodes.contains_key(code) {
            loaded_node.to_unload_wait_frame_count = 0;
        } else {
            loaded_node.to_unload_wait_frame_count += 1;
            if loaded_node.to_unload_wait_frame_count > 3 {
                to_unload_nodes.insert(*code, loaded_node.leaf_node_key);
            }
        }
    }

    loader
        .pending_load_leaf_node_map
        .extend(to_load_nodes.iter());

    loader
        .pending_unload_leaf_node_map
        .extend(to_unload_nodes.iter());

    for (_code, leaf_node_key) in loader.pending_reload_leaf_node_map.iter_mut() {
        update_leaf_node_data(
            leaf_node_key,
            &frustums,
            &global_transforms,
            &terrain_setting,
        );
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
        .filter(|(_, key)| loader.is_loaded(&key.address).not());

    to_load_nodes.for_each(|(code, _key)| {
        load_event.node_addresses.push(*code);
    });

    if load_event.node_addresses.is_empty() {
        return;
    }

    for address in load_event.node_addresses.iter() {
        let leaf_node_key = loader.pending_load_leaf_node_map.remove(address).unwrap();
        loader
            .loaded_leaf_node_map
            .insert(*address, LoadedNodeInfo::new(leaf_node_key));
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
        .filter(|(_, key)| (loader.can_unload(&key.address)));

    to_unload_nodes.for_each(|(code, _key)| {
        load_event.node_addresses.push(*code);
    });

    for address in load_event.node_addresses.iter() {
        loader.pending_unload_leaf_node_map.remove(address);
        loader.loaded_leaf_node_map.remove(address);
    }

    if load_event.node_addresses.is_empty() {
        return;
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
        .filter(|(_, key)| loader.can_reload(&key.address) && key.can_load());

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
