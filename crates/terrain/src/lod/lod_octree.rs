use std::hash::{Hash, Hasher};

use atom_utils::swap_data::{SwapData, SwapDataTakeTrait, SwapDataTrait};
/// TODO visibility range 范围就设置在chunk的创建和删除距离上。误差可以配置。（之后再做，chunk的加载和卸载可能有问题）
use bevy::{
    math::{bounding::Aabb3d, Vec3A},
    prelude::*,
    render::primitives::Frustum,
    utils::HashSet,
};
use bevy_console::{clap::Parser, AddConsoleCommand, ConsoleCommand};
use strum::IntoEnumIterator;

use crate::{
    setting::TerrainSetting, tables::SubNodeIndex, utils::OctreeUtil, TerrainObserver,
    TerrainSystemSet,
};

use super::morton_code::MortonCode;

pub type ObserverLocations = smallvec::SmallVec<[Vec3A; 1]>;
pub type LodOctreeDepthType = u8;
pub type ObserverFrustums = smallvec::SmallVec<[Frustum; 1]>;
pub type ObserverGlobalTransforms = smallvec::SmallVec<[GlobalTransform; 1]>;

#[derive(Debug, Default)]
pub struct TerrainLodOctreePlugin;

impl Plugin for TerrainLodOctreePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainLodOctree>()
            .add_systems(
                Update,
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

    pub fn find_leaf_node(&self, code: &MortonCode) -> Option<&TerrainLodOctreeNode> {
        self.get_current().get(&TerrainLodOctreeNode {
            code: *code,
            aabb: Aabb3d::new(Vec3A::ZERO, Vec3A::ZERO),
        })
    }

    pub fn remove_leaf_node(&mut self, node: &TerrainLodOctreeNode) {
        self.get_current_mut().remove(node);
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
        let level = code.depth() as usize;
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
                assert_eq!(level, child_node.code.depth());
                let can_divide = can_divide_node(
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

fn can_divide_node(
    node: &TerrainLodOctreeNode,
    current_level: LodOctreeDepthType,
    node_aabb: Aabb3d,
    observer_locations: &ObserverLocations,
    setting: &Res<TerrainSetting>,
) -> bool {
    if can_divide(node, observer_locations, node_aabb, setting)
        && current_level < setting.get_lod_octree_depth()
    {
        return true;
    }

    false
}

/// TODO depth小到一定程序就不再上移，否则相邻的chunk的mesh的连接，看起来过于粗糙，出现漏洞。
fn can_divide(
    node: &TerrainLodOctreeNode,
    observer_locations: &ObserverLocations,
    node_aabb: Aabb3d,
    setting: &Res<TerrainSetting>,
) -> bool {
    let mut max_depth = 0;
    for observer_location in observer_locations.iter() {
        let closest_point = node_aabb.closest_point(*observer_location);
        if closest_point == *observer_location {
            return true;
        }

        let distance = (closest_point - *observer_location).length().max(1.0);
        let distance = distance / setting.chunk_size;
        // 更小才能细分
        let clipmap_lod = distance.log(2.0) as LodOctreeDepthType;
        // 避免过大，导致overflow，出bug。
        let clipmap_lod = clipmap_lod.min(setting.get_lod_octree_depth());
        let clipmap_depth = setting.get_lod_octree_depth() - clipmap_lod;
        max_depth = max_depth.max(clipmap_depth);
    }
    // 更大才能细分
    node.code.depth < max_depth
}

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
