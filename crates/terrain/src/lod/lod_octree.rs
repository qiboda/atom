use bevy::{
    math::{
        bounding::{Aabb3d, BoundingVolume},
        Vec3A,
    },
    prelude::*,
    render::primitives::Frustum,
    utils::HashMap,
};
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::{
    chunk_mgr::chunk::chunk_lod::OctreeDepthType,
    isosurface::dc::octree::address::NodeAddress,
    setting::TerrainSetting,
    utils::{OctreeUtil, TerrainChunkUtils},
    TerrainObserver, TerrainSystemSet,
};

#[derive(Debug, Default)]
pub struct TerrainLodOctreePlugin;

impl Plugin for TerrainLodOctreePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LodOctreeMap>()
            .observe(trigger_lod_octree_node_added)
            .observe(trigger_lod_octree_node_removed)
            .add_systems(
                Update,
                update_lod_octree_nodes.in_set(TerrainSystemSet::UpdateLodOctree),
            )
            .add_systems(Update, log_lod_octree_nodes);
    }
}

#[derive(Debug, Resource, Default)]
pub struct LodOctreeMap {
    node_map: HashMap<NodeAddress, Entity>,
    leaf_node_map: HashMap<NodeAddress, Entity>,
}

impl LodOctreeMap {
    pub fn get_node_entity(&self, address: NodeAddress) -> Option<&Entity> {
        self.node_map.get(&address)
    }

    pub fn insert_node(
        &mut self,
        address: NodeAddress,
        entity: Entity,
        node_type: LodOctreeNodeType,
    ) {
        match node_type {
            LodOctreeNodeType::Leaf => {
                self.node_map.insert(address, entity);
                self.leaf_node_map.insert(address, entity);
            }
            LodOctreeNodeType::Internal => {
                self.node_map.insert(address, entity);
            }
        }
    }

    pub fn remove_node(&mut self, address: NodeAddress) {
        self.node_map.remove(&address);
        self.leaf_node_map.remove(&address);
    }
}

fn trigger_lod_octree_node_added(
    trigger: Trigger<OnAdd, LodOctreeNode>,
    query: Query<&LodOctreeNode>,
    mut lod_octree_map: ResMut<LodOctreeMap>,
) {
    let entity = trigger.entity();
    if let Ok(lod_octree_node) = query.get(entity) {
        lod_octree_map.insert_node(lod_octree_node.address, entity, lod_octree_node.node_type);
    }
}

fn trigger_lod_octree_node_removed(
    trigger: Trigger<OnRemove, LodOctreeNode>,
    query: Query<&LodOctreeNode>,
    mut lod_octree_map: ResMut<LodOctreeMap>,
) {
    let entity = trigger.entity();
    if let Ok(lod_octree_node) = query.get(entity) {
        lod_octree_map.remove_node(lod_octree_node.address);
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum LodOctreeNodeType {
    Leaf,
    Internal,
}

/// 所有节点扁平的挂在[`Terrain`]上。
/// [`Terrain`]: crate::
#[derive(Debug, Component)]
pub struct LodOctreeNode {
    // 深度越深，lod越小。
    pub address: NodeAddress,
    pub aabb: Aabb3d,
    pub node_type: LodOctreeNodeType,
}

type ObserverCoords = smallvec::SmallVec<[TerrainChunkCoord; 1]>;

fn update_lod_octree_node_top_to_bottom(
    commands: &mut Commands,
    aabb: Aabb3d,
    address: NodeAddress,
    observer_coords: &ObserverCoords,
    setting: &Res<TerrainSetting>,
    lod_octree_map: &mut ResMut<LodOctreeMap>,
    parent_entity: Entity,
) {
    let can_divide = can_divide_node(&address, &aabb, observer_coords, setting);

    let node_entity = lod_octree_map.node_map.get(&address);
    let new_node_entity = match node_entity {
        None => {
            let node_type = if can_divide {
                LodOctreeNodeType::Internal
            } else {
                LodOctreeNodeType::Leaf
            };
            commands
                .spawn(LodOctreeNode {
                    address,
                    aabb,
                    node_type,
                })
                .set_parent(parent_entity)
                .id()
        }
        Some(node_entity) => *node_entity,
    };

    if can_divide {
        for child_address in address.get_children_addresses() {
            let child_aabb =
                OctreeUtil::get_subnode_aabb(aabb, child_address.get_pos_in_parent().unwrap());
            update_lod_octree_node_top_to_bottom(
                commands,
                child_aabb,
                child_address,
                observer_coords,
                setting,
                lod_octree_map,
                new_node_entity,
            );
        }
    } else {
        commands.entity(new_node_entity).despawn_descendants();
    }
}

// 使用address遍历，会不知道是否还是子节点需要删除。
// 使用节点遍历，删除和添加很方便，但是不便于查找相邻的节点。
// 因此必须创建中间节点。以及节点类型判断支持。
fn update_lod_octree_nodes(
    mut commands: Commands,
    observer_query: Query<&GlobalTransform, With<TerrainObserver>>,
    root_node_query: Query<(Entity, &LodOctreeNode), Without<Parent>>,
    mut lod_octree_map: ResMut<LodOctreeMap>,
    setting: Res<TerrainSetting>,
) {
    if observer_query.iter().len() == 0 {
        return;
    }

    let mut observer_coords: ObserverCoords = smallvec::smallvec![];
    for observer_transform in observer_query.iter() {
        let coord = TerrainChunkUtils::get_coord_from_location(
            setting.chunk_setting.chunk_size,
            observer_transform.translation_vec3a(),
        );
        observer_coords.push(coord);
    }

    debug_assert!(root_node_query.iter().len() <= 1);

    if root_node_query.iter().len() == 0 {
        let root_address = NodeAddress::root();
        let lod_octree_size = setting.get_lod_octree_size();
        let root_aabb = Aabb3d::new(Vec3A::ZERO, Vec3A::splat(lod_octree_size * 0.5));

        let can_divide = can_divide_node(&root_address, &root_aabb, &observer_coords, &setting);

        let node_type = if can_divide {
            LodOctreeNodeType::Internal
        } else {
            LodOctreeNodeType::Leaf
        };
        let root_entity = commands
            .spawn(LodOctreeNode {
                address: root_address,
                aabb: root_aabb,
                node_type,
            })
            .id();

        if can_divide {
            for child_address in root_address.get_children_addresses() {
                let child_aabb = OctreeUtil::get_subnode_aabb(
                    root_aabb,
                    child_address.get_pos_in_parent().unwrap(),
                );
                update_lod_octree_node_top_to_bottom(
                    &mut commands,
                    child_aabb,
                    child_address,
                    &observer_coords,
                    &setting,
                    &mut lod_octree_map,
                    root_entity,
                );
            }
        } else {
            commands.entity(root_entity).despawn_descendants();
        }
    } else {
        let (root_entity, root_node) = root_node_query.get_single().expect("root node must one");

        if can_divide_node(
            &root_node.address,
            &root_node.aabb,
            &observer_coords,
            &setting,
        ) {
            for child_address in root_node.address.get_children_addresses() {
                let child_aabb = OctreeUtil::get_subnode_aabb(
                    root_node.aabb,
                    child_address.get_pos_in_parent().unwrap(),
                );
                update_lod_octree_node_top_to_bottom(
                    &mut commands,
                    child_aabb,
                    child_address,
                    &observer_coords,
                    &setting,
                    &mut lod_octree_map,
                    root_entity,
                );
            }
        } else {
            commands.entity(root_entity).despawn_descendants();
        }
    }
}

fn can_divide_node(
    node_address: &NodeAddress,
    node_aabb: &Aabb3d,
    observer_coords: &ObserverCoords,
    setting: &Res<TerrainSetting>,
) -> bool {
    let node_coord = TerrainChunkUtils::get_coord_from_location(
        setting.chunk_setting.chunk_size,
        node_aabb.center(),
    );
    let max_depth = get_node_max_depth(observer_coords, node_coord, setting);
    node_address.get_depth() < max_depth
        && node_address.get_depth() < setting.lod_setting.get_lod_octree_depth()
}

fn get_node_max_depth(
    frustum_coords: &ObserverCoords,
    node_coord: TerrainChunkCoord,
    setting: &Res<TerrainSetting>,
) -> OctreeDepthType {
    let mut max_depth = 0;
    for frustum_coord in frustum_coords.iter() {
        let chebyshev_distance = (*frustum_coord - node_coord).chebyshev_distance();
        let clipmap_depth = setting
            .lod_setting
            .get_depth(chebyshev_distance)
            .unwrap_or_else(|| {
                panic!(
                    "get invalid depth by chebyshev distance: {}",
                    chebyshev_distance
                )
            });
        max_depth = max_depth.max(clipmap_depth);
    }
    max_depth
}

fn log_lod_octree_nodes(
    lod_octree_map: Res<LodOctreeMap>,
    query: Query<&LodOctreeNode>,
    observer_query: Query<&GlobalTransform, With<TerrainObserver>>,
    setting: Res<TerrainSetting>,
) {
    // if observer_query.iter().len() == 0 {
    //     return;
    // }

    // let mut observer_coords: ObserverCoords = smallvec::smallvec![];
    // for observer_transform in observer_query.iter() {
    //     let coord = TerrainChunkUtils::get_coord_from_location(
    //         setting.chunk_setting.chunk_size,
    //         observer_transform.translation_vec3a(),
    //     );
    //     observer_coords.push(coord);
    // }

    // error!(
    //     "lod octree map size: {}, node size: {}",
    //     lod_octree_map.node_map.len(),
    //     query.iter().count()
    // );

    // for node in query.iter() {
    //     let node_coord = TerrainChunkUtils::get_coord_from_location(64.0, node.aabb.center());
    //     let max_depth = get_node_max_depth(&observer_coords, node_coord, &setting);
    //     error!(
    //         "node depth: {:?}, max_depth: {}",
    //         node.address.get_depth(),
    //         max_depth
    //     );
    // }
}
