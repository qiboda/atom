use std::collections::BinaryHeap;

// FIXME 有时候没有删除chunk内容。
use crate::{
    isosurface::dc::octree::address::NodeAddress,
    lod::lod_octree::{LodOctreeNode, LodOctreeNodeType},
    setting::TerrainSetting,
    TerrainObserver,
};
use bevy::{
    math::{bounding::BoundingVolume, Affine3A},
    prelude::*,
    render::{
        camera::CameraProjection,
        primitives::{Aabb, Frustum},
    },
    tasks::AsyncComputeTaskPool,
    utils::HashSet,
};
use smallvec::SmallVec;

use super::{
    chunk::{bundle::TerrainChunk, state::TerrainChunkState},
    TerrainChunkSystemSet,
};

#[derive(Debug, Default)]
pub struct TerrainChunkLoaderPlugin;

impl Plugin for TerrainChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TerrainChunkLoadEvent>()
            .add_event::<TerrainChunkUnLoadEvent>()
            .init_state::<TerrainCreateState>()
            .init_resource::<TerrainChunkLoader>()
            .observe(trigger_lod_node_remove)
            .add_systems(
                Update,
                (
                    update_terrain_create_state,
                    to_unload_chunk,
                    update_loading_data,
                    to_load_chunk,
                )
                    .chain()
                    .in_set(TerrainChunkSystemSet::UpdateLoader),
            );
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct LeafNodeKey {
    pub address: NodeAddress,
    pub entity: Entity,

    pub is_in_frustums: bool,
    pub distance: u64,
    // unit is degree
    pub angle: u32,
}
impl LeafNodeKey {
    fn new(leaf_node_entity: Entity) -> Self {
        Self {
            address: NodeAddress::root(),
            entity: leaf_node_entity,
            is_in_frustums: false,
            distance: 0,
            angle: 0,
        }
    }
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
        match self.distance.partial_cmp(&other.distance) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.angle.partial_cmp(&other.angle)
    }
}

#[derive(Resource, Debug, Default)]
pub struct TerrainChunkLoader {
    pub leaf_node_pending_load_deque: BinaryHeap<LeafNodeKey>,
    pub loaded_leaf_node_set: HashSet<NodeAddress>,
    pub pending_unload_leaf_node_set: HashSet<NodeAddress>,
}

impl TerrainChunkLoader {
    pub fn is_loaded(&self, node_address: &NodeAddress) -> bool {
        self.loaded_leaf_node_set.contains(node_address)
    }
}

type FrustumsType = SmallVec<[Frustum; 1]>;
type GlobalTransformsType = SmallVec<[GlobalTransform; 1]>;

fn update_leaf_node_data(
    leaf_node_key: &mut LeafNodeKey,
    leaf_node: &LodOctreeNode,
    frustums: &FrustumsType,
    global_transforms: &GlobalTransformsType,
) {
    let aabb = Aabb::from_min_max(leaf_node.aabb.min.into(), leaf_node.aabb.max.into());

    let mut is_in_frustums = false;
    for frustum in frustums.iter() {
        if frustum.intersects_obb(&aabb, &Affine3A::IDENTITY, true, true) {
            is_in_frustums = true;
            break;
        }
    }
    let mut min_distance = u64::MAX;
    let mut min_angle = u32::MAX;
    for global_transform in global_transforms.iter() {
        let (_, rotation, translation) = global_transform.to_scale_rotation_translation();
        let leaf_node_location: Vec3 = leaf_node.aabb.center().into();
        let relative_translation = leaf_node_location - translation;
        min_distance = min_distance.min(relative_translation.length_squared() as u64);

        let (axis, _angle) = rotation.to_axis_angle();
        min_angle = min_angle.min(relative_translation.angle_between(axis).to_degrees() as u32);
    }

    leaf_node_key.is_in_frustums = is_in_frustums;
    leaf_node_key.distance = min_distance;
    leaf_node_key.angle = min_angle;
}

#[allow(clippy::type_complexity)]
pub fn update_loading_data(
    observer_query: Query<
        (&GlobalTransform, &Projection),
        (Changed<GlobalTransform>, With<TerrainObserver>),
    >,
    mut loader: ResMut<TerrainChunkLoader>,
    node_query: Query<(Entity, &LodOctreeNode)>,
) {
    let _span = info_span!("update_loading_data").entered();

    let mut frustums = FrustumsType::new();
    let mut global_transforms = GlobalTransformsType::new();
    for (global_transform, projection) in observer_query.iter() {
        let frustum = projection.compute_frustum(global_transform);
        frustums.push(frustum);
        global_transforms.push(*global_transform);
    }

    loader.leaf_node_pending_load_deque.clear();
    for (entity, node) in node_query.iter() {
        if node.node_type == LodOctreeNodeType::Internal {
            if loader.is_loaded(&node.address) {
                loader.pending_unload_leaf_node_set.insert(node.address);
            }
            continue;
        }

        if loader.is_loaded(&node.address) {
            continue;
        }
        loader.pending_unload_leaf_node_set.remove(&node.address);

        let mut leaf_node_key = LeafNodeKey::new(entity);
        update_leaf_node_data(&mut leaf_node_key, node, &frustums, &global_transforms);
        loader.leaf_node_pending_load_deque.push(leaf_node_key);
    }
}

pub fn trigger_lod_node_remove(
    trigger: Trigger<OnRemove, LodOctreeNode>,
    query: Query<&LodOctreeNode>,
    mut loader: ResMut<TerrainChunkLoader>,
) {
    let node_entity = trigger.entity();
    if let Ok(node) = query.get(node_entity) {
        loader.pending_unload_leaf_node_set.insert(node.address);
    };
}

pub fn to_load_chunk(
    query: Query<&LodOctreeNode>,
    mut terrain_chunk_loader: ResMut<TerrainChunkLoader>,
    mut event_writer: EventWriter<TerrainChunkLoadEvent>,
    setting: Res<TerrainSetting>,
    state: Res<State<TerrainCreateState>>,
) {
    if *state == TerrainCreateState::AllSeamMeshCreateEnd {
        let num_per_core = setting.lod_setting.load_node_num_per_processor_core as usize;
        // let num_per_core = 100000000;
        let num = num_per_core * AsyncComputeTaskPool::get().thread_num();
        let num = num.min(terrain_chunk_loader.leaf_node_pending_load_deque.len());
        for _ in 0..num {
            if let Some(key) = terrain_chunk_loader.leaf_node_pending_load_deque.pop() {
                if let Ok(lod_octree_node) = query.get(key.entity) {
                    info!("to load lod octree node: {:?}", lod_octree_node.address);
                    event_writer.send(TerrainChunkLoadEvent {
                        node_address: lod_octree_node.address,
                    });
                    terrain_chunk_loader
                        .loaded_leaf_node_set
                        .insert(lod_octree_node.address);
                }
            } else {
                break;
            }
        }
    }
}

pub fn to_unload_chunk(
    mut terrain_chunk_loader: ResMut<TerrainChunkLoader>,
    mut event_writer: EventWriter<TerrainChunkUnLoadEvent>,
    state: Res<State<TerrainCreateState>>,
) {
    if *state == TerrainCreateState::AllSeamMeshCreateEnd
        && terrain_chunk_loader.leaf_node_pending_load_deque.is_empty()
    {
        let unloaded_nodes = std::mem::take(&mut terrain_chunk_loader.pending_unload_leaf_node_set);
        for node_address in unloaded_nodes.iter() {
            event_writer.send(TerrainChunkUnLoadEvent {
                node_address: *node_address,
            });
            terrain_chunk_loader
                .loaded_leaf_node_set
                .remove(node_address);
        }

        terrain_chunk_loader.pending_unload_leaf_node_set.clear();
    }
}

#[derive(Event, Debug)]
pub struct TerrainChunkLoadEvent {
    pub node_address: NodeAddress,
}

#[derive(Event, Debug)]
pub struct TerrainChunkUnLoadEvent {
    pub node_address: NodeAddress,
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TerrainCreateState {
    #[default]
    ExistMainMeshCreating,
    AllMainMeshCreateEnd,
    ExistSeamMeshCreating,
    AllSeamMeshCreateEnd,
}

pub fn update_terrain_create_state(
    query: Query<&TerrainChunkState, With<TerrainChunk>>,
    mut terrain_create_state: ResMut<NextState<TerrainCreateState>>,
) {
    let mut terrain_chunk_state = TerrainChunkState::Done;
    for chunk_state in query.iter() {
        terrain_chunk_state = terrain_chunk_state.min(*chunk_state);
    }

    match terrain_chunk_state {
        TerrainChunkState::CreateMainMesh => {
            terrain_create_state.set(TerrainCreateState::ExistMainMeshCreating)
        }
        TerrainChunkState::WaitToCreateSeam => {
            terrain_create_state.set(TerrainCreateState::AllMainMeshCreateEnd)
        }
        TerrainChunkState::CreateSeamMesh => {
            terrain_create_state.set(TerrainCreateState::ExistSeamMeshCreating)
        }
        TerrainChunkState::Done => {
            terrain_create_state.set(TerrainCreateState::AllSeamMeshCreateEnd)
        }
    }
}
