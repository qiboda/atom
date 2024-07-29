use std::hash::{Hash, Hasher};

use atom_utils::swap_data::{SwapData, SwapDataTakeTrait, SwapDataTrait};
/// TODO visibility range 范围就设置在chunk的创建和删除距离上。误差可以配置。（之后再做，chunk的加载和卸载可能有问题）
use bevy::{
    math::{
        bounding::{Aabb3d, BoundingVolume},
        Vec3A,
    },
    prelude::*,
    utils::HashSet,
};
use bevy_console::{clap::Parser, AddConsoleCommand, ConsoleCommand};
use strum::IntoEnumIterator;

use crate::{
    lod::neighbor_query::{
        get_face_neighbor_lod_octree_nodes, get_neighbor_all_nodes,
        get_neighbor_negative_direction_nodes,
    },
    setting::TerrainSetting,
    tables::{FaceIndex, SubNodeIndex},
    utils::OctreeUtil,
    TerrainObserver, TerrainSystemSet,
};

use super::{
    lod_gizmos::TerrainLodGizmosPlugin, morton_code::MortonCode,
    neighbor_query::get_neighbor_positive_direction_nodes,
};

pub type ObserverLocations = smallvec::SmallVec<[Vec3A; 1]>;
pub type LodOctreeLevelType = u8;

#[derive(Debug, Default)]
pub struct TerrainLodOctreePlugin;

impl Plugin for TerrainLodOctreePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerrainLodGizmosPlugin)
            .init_resource::<TerrainLodOctree>()
            .add_systems(
                Update,
                // balance_terrain_lod_octree
                (update_terrain_lod_octree)
                    //
                    .chain()
                    .in_set(TerrainSystemSet::UpdateLodOctree),
            )
            .add_console_command::<TerrainLodLeafNodeNumCommand, _>(
                terrain_lod_leaf_node_num_command,
            );
    }
}

/// NOTE: code是真正的各种操作的key。aabb仅仅是一个附带的数据。
#[derive(Debug, Clone)]
pub struct TerrainLodOctreeNode {
    pub code: MortonCode,
    pub aabb: Aabb3d,
}

impl Hash for TerrainLodOctreeNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.code.hash(state);
    }
}

impl PartialEq for TerrainLodOctreeNode {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl Eq for TerrainLodOctreeNode {}

impl TerrainLodOctreeNode {
    pub fn get_child_node(&self, subnode_index: SubNodeIndex) -> TerrainLodOctreeNode {
        let child_aabb = OctreeUtil::get_subnode_aabb(self.aabb, subnode_index.to_index() as u8);
        TerrainLodOctreeNode {
            code: self.code.child(subnode_index).unwrap(),
            aabb: child_aabb,
        }
    }
}
#[derive(Debug, Default)]
pub struct TerrainLodOctreeLevel {
    pub leaf_nodes_a: HashSet<TerrainLodOctreeNode>,
    pub leaf_nodes_b: HashSet<TerrainLodOctreeNode>,
    // default a is new, b is old.
    pub b_is_new: bool,
}

impl TerrainLodOctreeLevel {
    pub fn swap(&mut self) {
        self.b_is_new = !self.b_is_new;
        if self.b_is_new {
            self.leaf_nodes_b.clear();
        } else {
            self.leaf_nodes_a.clear();
        }
    }

    pub fn get_current(&self) -> &HashSet<TerrainLodOctreeNode> {
        if self.b_is_new {
            &self.leaf_nodes_b
        } else {
            &self.leaf_nodes_a
        }
    }

    pub fn get_current_mut(&mut self) -> &mut HashSet<TerrainLodOctreeNode> {
        if self.b_is_new {
            &mut self.leaf_nodes_b
        } else {
            &mut self.leaf_nodes_a
        }
    }

    pub fn insert_leaf_node(&mut self, node: TerrainLodOctreeNode) {
        if self.b_is_new {
            self.leaf_nodes_b.insert(node);
        } else {
            self.leaf_nodes_a.insert(node);
        }
    }

    pub fn get_added_nodes(&self) -> impl Iterator<Item = &TerrainLodOctreeNode> + '_ {
        if self.b_is_new {
            self.leaf_nodes_b.difference(&self.leaf_nodes_a)
        } else {
            self.leaf_nodes_a.difference(&self.leaf_nodes_b)
        }
    }

    pub fn get_removed_nodes(&self) -> impl Iterator<Item = &TerrainLodOctreeNode> + '_ {
        if self.b_is_new {
            self.leaf_nodes_a.difference(&self.leaf_nodes_b)
        } else {
            self.leaf_nodes_b.difference(&self.leaf_nodes_a)
        }
    }

    fn find_leaf_node(&self, code: &MortonCode) -> Option<&TerrainLodOctreeNode> {
        self.get_current().get(&TerrainLodOctreeNode {
            code: *code,
            aabb: Aabb3d::new(Vec3A::ZERO, Vec3A::ZERO),
        })
    }

    fn remove_leaf_node(&mut self, node: &TerrainLodOctreeNode) {
        let removed = self.get_current_mut().remove(node);
    }
}

/// 使用松散八叉树不合适，因为缝隙填充需要关联多个相邻的chunk。
/// 使用松散八叉树，会导致缝隙填充的chunk的lod误差变大，缝隙填充的性能变差。
#[derive(Debug, Resource, Default)]
pub struct TerrainLodOctree {
    pub octree_levels: Vec<TerrainLodOctreeLevel>,
}

impl TerrainLodOctree {
    pub fn get_node(&self, code: &MortonCode) -> Option<&TerrainLodOctreeNode> {
        let level = code.level() as usize;
        if level >= self.octree_levels.len() {
            return None;
        }
        self.octree_levels[level].find_leaf_node(code)
    }
}

pub fn update_terrain_lod_octree(
    mut terrain_lod_octree: ResMut<TerrainLodOctree>,
    terrain_setting: Res<TerrainSetting>,
    observer_query: Query<&GlobalTransform, With<TerrainObserver>>,
) {
    // let _span = info_span!("update_terrain_lod_octree").entered();

    // 可能扩大，但不会缩小，为了支持热更新。避免没有删除chunk。
    let lod_octree_depth = terrain_setting.get_lod_octree_depth();
    let max_level = lod_octree_depth;
    if terrain_lod_octree.octree_levels.len() <= max_level as usize {
        terrain_lod_octree
            .octree_levels
            .resize_with((max_level + 1) as usize, TerrainLodOctreeLevel::default);
    }

    let mut observer_locations: ObserverLocations = smallvec::smallvec![];
    for observer_transform in observer_query.iter() {
        observer_locations.push(observer_transform.translation_vec3a());
    }

    // 清空旧数据。
    for level in terrain_lod_octree.octree_levels.iter_mut() {
        level.swap();
    }

    // 如果没有观察者，不需要更新。
    if observer_locations.is_empty() {
        return;
    }

    let mut can_divide_nodes_data: SwapData<Vec<TerrainLodOctreeNode>> = SwapData::default();

    let terrain_size = terrain_setting.get_terrain_size();
    let root_morton_code = MortonCode::encode(UVec3::new(0, 0, 0), 0);
    let root_node = TerrainLodOctreeNode {
        code: root_morton_code,
        aabb: Aabb3d::new(Vec3A::splat(0.0), Vec3A::splat(terrain_size * 0.5)),
    };
    if can_divide_node(
        &terrain_lod_octree,
        &root_node,
        0,
        root_node.aabb,
        &observer_locations,
        &terrain_setting,
    ) {
        can_divide_nodes_data.insert(root_node);
    } else {
        terrain_lod_octree.octree_levels[0].insert_leaf_node(root_node);
    }

    can_divide_nodes_data.swap();

    for level in 1..=max_level {
        for node in can_divide_nodes_data.take_last().iter() {
            for subnode_index in SubNodeIndex::iter() {
                let child_node = node.get_child_node(subnode_index);
                assert_eq!(level, child_node.code.level());
                let can_divide = can_divide_node(
                    &terrain_lod_octree,
                    &child_node,
                    level,
                    child_node.aabb,
                    &observer_locations,
                    &terrain_setting,
                );
                if can_divide {
                    can_divide_nodes_data.insert(child_node);
                } else {
                    terrain_lod_octree.octree_levels[level as usize].insert_leaf_node(child_node);
                }
            }
        }

        if can_divide_nodes_data.get_current().is_empty() {
            break;
        }

        can_divide_nodes_data.swap();
    }
}

pub struct BalanceNodeInfo {
    pub nodes: Vec<TerrainLodOctreeNode>,
    pub max_level: u8,
}

/// 从最深处向上逐级处理。
/// 找到相邻节点中跨度大于1的节点，删除节点，并细分他们，插入到更深的层次。
fn balance_terrain_lod_octree(mut terrain_lod_octree: ResMut<TerrainLodOctree>) {
    loop {
        let mut can_loop = false;
        for i in (2..terrain_lod_octree.octree_levels.len()).rev() {
            let mut to_divide_nodes = vec![];
            {
                let level = &terrain_lod_octree.octree_levels[i];
                for node in level.get_current().iter() {
                    let mut neighbor_nodes =
                        get_neighbor_positive_direction_nodes(&terrain_lod_octree, node, 10);
                    let mut max_level = node.code.level();
                    for neighbor_node in neighbor_nodes.iter() {
                        max_level = max_level.max(neighbor_node.code.level());
                    }
                    neighbor_nodes
                        .retain(|neighbor_node| max_level - neighbor_node.code.level() > 1);
                    if !neighbor_nodes.is_empty() {
                        let mut nodes: Vec<TerrainLodOctreeNode> =
                            neighbor_nodes.iter().map(|x| (*x).clone()).collect();
                        if max_level - node.code.level() > 1 {
                            nodes.push(node.clone());
                        }
                        to_divide_nodes.push(BalanceNodeInfo { nodes, max_level });
                    }
                }
            }

            for node_info in to_divide_nodes {
                can_loop = true;
                let max_level = node_info.max_level;
                for node in node_info.nodes.iter() {
                    if max_level - node.code.level() == 2 {
                        let current_level = (node.code.level() + 1) as usize;
                        for octant in SubNodeIndex::iter() {
                            let child_node = node.get_child_node(octant);
                            assert_eq!(child_node.code.level() as usize, current_level);
                            terrain_lod_octree.octree_levels[current_level]
                                .insert_leaf_node(child_node);
                        }
                    } else if max_level - node.code.level() == 3 {
                        for octant in SubNodeIndex::iter() {
                            let current_level = (node.code.level() + 2) as usize;
                            let child_node = node.get_child_node(octant);
                            assert_eq!(child_node.code.level() as usize, current_level - 1);
                            for octant_2 in SubNodeIndex::iter() {
                                let child_node = child_node.get_child_node(octant_2);
                                assert_eq!(child_node.code.level() as usize, current_level);
                                terrain_lod_octree.octree_levels[current_level]
                                    .insert_leaf_node(child_node);
                            }
                        }
                    }

                    assert!(max_level - node.code.level() <= 3);

                    terrain_lod_octree.octree_levels[node.code.level() as usize]
                        .remove_leaf_node(node);
                }
            }
        }

        if can_loop == false {
            return;
        }
    }
}

fn can_divide_node(
    terrain_lod_octree: &TerrainLodOctree,
    node: &TerrainLodOctreeNode,
    current_level: LodOctreeLevelType,
    node_aabb: Aabb3d,
    observer_locations: &ObserverLocations,
    setting: &Res<TerrainSetting>,
) -> bool {
    if can_divide(
        terrain_lod_octree,
        node,
        observer_locations,
        node_aabb,
        setting,
    ) && current_level < setting.get_lod_octree_depth()
    {
        return true;
    }

    return false;
    // let theory_depth = get_node_theory_depth(observer_locations, node_aabb, setting);
    // current_level < theory_depth && current_level < setting.get_lod_octree_depth()
}

fn can_divide(
    terrain_lod_octree: &TerrainLodOctree,
    node: &TerrainLodOctreeNode,
    observer_locations: &ObserverLocations,
    node_aabb: Aabb3d,
    setting: &Res<TerrainSetting>,
) -> bool {
    // for observer_location in observer_locations.iter() {
    //     if node_aabb.closest_point(*observer_location) == *observer_location {
    //         return true;
    //     }
    // }

    // let neighbor_nodes = get_neighbor_positive_direction_nodes(terrain_lod_octree, node, 2);
    // let mut max_level = 0;
    // for neighbor_node in neighbor_nodes.iter() {
    //     max_level = max_level.max(neighbor_node.code.level());
    // }
    // if max_level != 0 && max_level - node.code.level() >= 1 {
    //     return false;
    // }

    let mut max_depth = 0;
    for observer_location in observer_locations.iter() {
        let closest_point = node_aabb.closest_point(*observer_location);
        if closest_point == *observer_location {
            return true;
        }

        let distance = (closest_point - *observer_location).length().max(1.0);
        let distance = distance / setting.chunk_size;
        // 更小才能细分
        let clipmap_lod = distance.log(2.0) as LodOctreeLevelType;
        // 避免过大，导致overflow，出bug。
        let clipmap_lod = clipmap_lod.min(setting.get_lod_octree_depth());
        let clipmap_depth = setting.get_lod_octree_depth() - clipmap_lod;
        max_depth = max_depth.max(clipmap_depth);
    }
    // 更大才能细分
    node.code.level < max_depth
}

// fn get_node_theory_depth(
//     observer_locations: &ObserverLocations,
//     node_aabb: Aabb3d,
//     setting: &Res<TerrainSetting>,
// ) -> LodOctreeLevelType {
//     let mut max_depth = 0;

//     let node_aabb_double = node_aabb
//         .scale_around_center(Vec3A::splat(2.0))
//         .grow(Vec3A::splat(1.0));
//     let node_chunk_coord = (node_aabb.min / setting.chunk_size).floor();
//     for observer_location in observer_locations.iter() {
//         if node_aabb_double.closest_point(observer_location) == observer_location {
//             continue;
//         }

//         // let observer_chunk_coord = (*observer_location / setting.chunk_size).floor();
//         // let chebyshev_distance = (node_chunk_coord - observer_chunk_coord)
//         //     .abs()
//         //     .max_element();
//         // // 更小才能细分
//         // let clipmap_lod = chebyshev_distance.max(1.0).log(2.0) as LodOctreeLevelType;
//         // // 避免过大，导致overflow，出bug。
//         // let clipmap_lod = clipmap_lod.min(setting.get_lod_octree_depth());
//         // debug!(
//         //     "node_chunk_coord: {:?}, location: {:?}, observer chunk coord: {:?}, location:{:?}, distance:{}, lod:{}",
//         //     node_chunk_coord,
//         //     node_aabb.center(),
//         //     observer_chunk_coord,
//         //     observer_location,
//         //     chebyshev_distance,
//         //     clipmap_lod,
//         // );
//         let clipmap_depth = setting.get_lod_octree_depth() - clipmap_lod;
//         max_depth = max_depth.max(clipmap_depth);
//     }
//     // 更大才能细分
//     max_depth
// }

#[derive(Parser, ConsoleCommand)]
#[command(name = "terrain.lod.octree.node_num")]
pub struct TerrainLodLeafNodeNumCommand;

fn terrain_lod_leaf_node_num_command(
    mut persist: ConsoleCommand<TerrainLodLeafNodeNumCommand>,
    lod_octree: Res<TerrainLodOctree>,
) {
    if let Some(Ok(_cmd)) = persist.take() {
        lod_octree
            .octree_levels
            .iter()
            .enumerate()
            .for_each(|(i, level)| {
                info!("level {}: {}", i, level.get_current().len());
            });

        persist.ok();
    }
}
