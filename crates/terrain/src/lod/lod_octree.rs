/// TODO 进入节点，创建子节点，修改当前节点到中间节点，这立即删除了旧的叶子节点，并创建了新的。
/// TODO 离开节点，不立即删除，检测离开范围超过一半时再删除，
/// TODO 也就是新增叶子节点就立即执行，
/// TODO 删除叶子节点延迟执行。
///
/// TODO visibility range 范围就设置在chunk的创建和删除距离上。误差可以配置。（之后再做，chunk的加载和卸载可能有问题）
/// TODO 需要标记那些chunk要删除，把他们从缝隙创建中排除出去。
use bevy::{
    math::{
        bounding::{Aabb3d, BoundingVolume},
        Vec3A,
    },
    prelude::*,
    utils::HashMap,
};

use crate::{
    chunk_mgr::chunk::chunk_lod::OctreeDepthType, isosurface::dc::octree::address::NodeAddress,
    setting::TerrainSetting, utils::OctreeUtil, TerrainObserver, TerrainSystemSet,
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
                (
                    update_lod_octree_nodes,
                    // apply_deferred,
                    // check_lod_octree_node_type,
                )
                    .chain()
                    .in_set(TerrainSystemSet::UpdateLodOctree),
            )
            .add_systems(Update, log_lod_octree_nodes);
    }
}

#[derive(Debug, Resource, Default)]
pub struct LodOctreeMap {
    pub node_map: HashMap<NodeAddress, Entity>,
}

impl LodOctreeMap {
    pub fn get_node_entity(&self, address: NodeAddress) -> Option<&Entity> {
        self.node_map.get(&address)
    }

    pub fn insert_node(&mut self, address: NodeAddress, entity: Entity) {
        self.node_map.insert(address, entity);
    }

    pub fn remove_node(&mut self, address: NodeAddress) {
        self.node_map.remove(&address);
    }
}

fn trigger_lod_octree_node_added(
    trigger: Trigger<OnAdd, LodOctreeNode>,
    query: Query<&LodOctreeNode>,
    mut lod_octree_map: ResMut<LodOctreeMap>,
) {
    let entity = trigger.entity();
    if let Ok(lod_octree_node) = query.get(entity) {
        lod_octree_map.insert_node(lod_octree_node.address, entity);
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

type ObserverLocations = smallvec::SmallVec<[Vec3A; 1]>;

#[allow(clippy::too_many_arguments)]
fn update_lod_octree_node_top_to_bottom(
    commands: &mut Commands,
    aabb: Aabb3d,
    address: NodeAddress,
    observer_locations: &ObserverLocations,
    setting: &Res<TerrainSetting>,
    lod_octree_map: &mut ResMut<LodOctreeMap>,
    parent_entity: Entity,
    children_node_query: &mut Query<&mut LodOctreeNode, With<Parent>>,
) {
    let node_entity = lod_octree_map.node_map.get(&address);
    let (parent_node_entity, can_divide) = match node_entity {
        None => {
            let can_divide = can_divide_node(
                &address,
                &aabb,
                observer_locations,
                setting,
                LodOctreeNodeUpdateType::Add,
            );
            let node_type = if can_divide {
                LodOctreeNodeType::Internal
            } else {
                LodOctreeNodeType::Leaf
            };
            let node_entity = commands
                .spawn(LodOctreeNode {
                    address,
                    aabb,
                    node_type,
                })
                .set_parent(parent_entity)
                .id();
            (node_entity, can_divide)
        }
        Some(node_entity) => {
            let can_divide = can_divide_node(
                &address,
                &aabb,
                observer_locations,
                setting,
                LodOctreeNodeUpdateType::Remove,
            );
            if let Ok(mut node) = children_node_query.get_mut(*node_entity) {
                if can_divide {
                    node.node_type = LodOctreeNodeType::Internal;
                } else {
                    node.node_type = LodOctreeNodeType::Leaf;
                    commands.entity(*node_entity).despawn_descendants();
                }
            }
            (*node_entity, can_divide)
        }
    };

    if can_divide {
        for child_address in address.get_children_addresses() {
            let child_aabb =
                OctreeUtil::get_subnode_aabb(aabb, child_address.get_pos_in_parent().unwrap());
            update_lod_octree_node_top_to_bottom(
                commands,
                child_aabb,
                child_address,
                observer_locations,
                setting,
                lod_octree_map,
                parent_node_entity,
                children_node_query,
            );
        }
    }
}

// 使用address遍历，会不知道是否还是子节点需要删除。
// 使用节点遍历，删除和添加很方便，但是不便于查找相邻的节点。
// 因此必须创建中间节点。以及节点类型判断支持。
fn update_lod_octree_nodes(
    mut commands: Commands,
    observer_query: Query<&GlobalTransform, With<TerrainObserver>>,
    mut root_node_query: Query<(Entity, &mut LodOctreeNode), Without<Parent>>,
    mut children_node_query: Query<&mut LodOctreeNode, With<Parent>>,
    mut lod_octree_map: ResMut<LodOctreeMap>,
    setting: Res<TerrainSetting>,
) {
    let _span = info_span!("update_lod_octree_nodes").entered();

    if observer_query.iter().len() == 0 {
        if let Ok((entity, _node)) = root_node_query.get_single() {
            if let Some(entity_cmds) = commands.get_entity(entity) {
                entity_cmds.despawn_recursive();
            }
        }
        return;
    }

    let mut observer_locations: ObserverLocations = smallvec::smallvec![];
    for observer_transform in observer_query.iter() {
        observer_locations.push(observer_transform.translation_vec3a());
    }

    debug_assert!(root_node_query.iter().len() <= 1);

    if root_node_query.iter().len() == 0 {
        let root_address = NodeAddress::root();
        let lod_octree_size = setting.get_lod_octree_size();
        let root_aabb = Aabb3d::new(Vec3A::ZERO, Vec3A::splat(lod_octree_size * 0.5));

        let can_divide = can_divide_node(
            &root_address,
            &root_aabb,
            &observer_locations,
            &setting,
            LodOctreeNodeUpdateType::Add,
        );

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
                    &observer_locations,
                    &setting,
                    &mut lod_octree_map,
                    root_entity,
                    &mut children_node_query,
                );
            }
        } else {
            commands.entity(root_entity).despawn_descendants();
        }
    } else {
        let (root_entity, mut root_node) = root_node_query
            .get_single_mut()
            .expect("root node must one");

        if can_divide_node(
            &root_node.address,
            &root_node.aabb,
            &observer_locations,
            &setting,
            LodOctreeNodeUpdateType::Remove,
        ) {
            root_node.node_type = LodOctreeNodeType::Internal;
            for child_address in root_node.address.get_children_addresses() {
                let child_aabb = OctreeUtil::get_subnode_aabb(
                    root_node.aabb,
                    child_address.get_pos_in_parent().unwrap(),
                );
                update_lod_octree_node_top_to_bottom(
                    &mut commands,
                    child_aabb,
                    child_address,
                    &observer_locations,
                    &setting,
                    &mut lod_octree_map,
                    root_entity,
                    &mut children_node_query,
                );
            }
        } else {
            commands.entity(root_entity).despawn_descendants();
            root_node.node_type = LodOctreeNodeType::Leaf;
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LodOctreeNodeUpdateType {
    Add,
    Remove,
}

fn can_divide_node(
    node_address: &NodeAddress,
    node_aabb: &Aabb3d,
    observer_locations: &ObserverLocations,
    setting: &Res<TerrainSetting>,
    update_type: LodOctreeNodeUpdateType,
) -> bool {
    let max_depth = get_node_theory_depth(observer_locations, node_aabb, setting, update_type);
    node_address.get_depth() < max_depth
        && node_address.get_depth() < setting.lod_setting.get_lod_octree_depth()
}

fn get_node_theory_depth(
    observer_locations: &ObserverLocations,
    node_aabb: &Aabb3d,
    setting: &Res<TerrainSetting>,
    update_type: LodOctreeNodeUpdateType,
) -> OctreeDepthType {
    let mut max_depth = 0;
    let center = node_aabb.center();
    for observer_location in observer_locations.iter() {
        let distance = (*observer_location - center).length() * 0.75;
        let mut chunk_distance = distance / setting.chunk_setting.chunk_size;
        // let old_clipmap_lod = chunk_distance.log2().floor() as OctreeDepthType;
        if update_type == LodOctreeNodeUpdateType::Remove {
            chunk_distance -= 0.5;
        }
        // 更小才能细分
        let clipmap_lod = chunk_distance.log2().floor() as OctreeDepthType;
        // error!(
        //     "old_clipmap_lod: {}, clipmap_lod: {}, diff: {}",
        //     old_clipmap_lod,
        //     clipmap_lod,
        //     clipmap_lod - old_clipmap_lod
        // );
        let clipmap_depth = setting.lod_setting.get_lod_octree_depth() - clipmap_lod;
        max_depth = max_depth.max(clipmap_depth);
    }
    // 更大才能细分
    max_depth
}

fn log_lod_octree_nodes(
    lod_octree_map: Res<LodOctreeMap>,
    query: Query<&LodOctreeNode>,
    observer_query: Query<&GlobalTransform, With<TerrainObserver>>,
) {
    if observer_query.iter().len() == 0 {
        return;
    }

    let mut leaf_num = 0;
    let mut internal_num = 0;
    for node in query.iter() {
        if node.node_type == LodOctreeNodeType::Leaf {
            leaf_num += 1;
        } else {
            internal_num += 1;
        }
    }

    info!(
        "lod octree map size: {}, node size: {}, leaf num: {}, internal num: {}",
        lod_octree_map.node_map.len(),
        query.iter().count(),
        leaf_num,
        internal_num,
    );
}
