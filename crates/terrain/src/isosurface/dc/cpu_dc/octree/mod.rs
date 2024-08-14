pub mod node;

use std::sync::{Arc, RwLock};

use bevy::{
    math::{bounding::Aabb3d, Vec3A},
    prelude::*,
    utils::{tracing::instrument, HashMap},
};
use ndshape::Shape;

use crate::{
    chunk_mgr::chunk::comp::TerrainChunkBorderVertices,
    lod::morton_code::MortonCode,
    tables::{SubNodeIndex, VertexIndex},
    utils::OctreeUtil,
};
use node::{Node, NodeType};
use pqef::quadric::Quadric;
use strum::{EnumCount, IntoEnumIterator};

pub trait OctreeSampler {
    fn sampler(&self, loc: Vec3) -> f32;
    fn sampler_split(&self, x: f32, y: f32, z: f32) -> f32;
}

/// octree
/// 1. 细分octree，是否可以细分
/// 2. 计算叶子的qef。
/// 3. 反向进行收缩，节省内容空间。
///
/// 自顶向下，还是自底向上？
///
/// 也许可以通过不断插入顶点，来决定是否应该细分。()
/// 性能比每次细分来检测所有的顶点数据要好（层级细分是层级数*(每层涵盖的顶点数 = 所有顶点数）)，

#[derive()]
pub struct OctreeProxy<'a> {
    pub octree: &'a Octree,
    pub is_seam: bool,
}

impl<'a> OctreeProxy<'a> {
    pub fn get_node(&self, address: &MortonCode) -> Option<&Node> {
        self.octree.get_node(address)
    }
}

#[derive(Default, Debug)]
pub struct OctreeLevel {
    pub address_node_map: HashMap<MortonCode, Node>,
}

#[derive(Component)]
pub struct Octree {
    pub levels: Vec<OctreeLevel>,
    pub node_shape: ndshape::RuntimeShape<u32, 3>,
}

impl Octree {
    pub fn new(node_shape: ndshape::RuntimeShape<u32, 3>) -> Self {
        Octree {
            levels: vec![],
            node_shape,
        }
    }

    pub fn get_nodes_num(&self) -> usize {
        let mut num = 0;
        for level in self.levels.iter() {
            num += level.address_node_map.len();
        }
        num
    }

    pub fn get_node(&self, address: &MortonCode) -> Option<&Node> {
        let level = address.depth() as usize;
        if self.levels.len() <= level {
            return None;
        }
        self.levels[level].address_node_map.get(address)
    }

    pub fn insert_leaf_node(&mut self, node: Node) {
        let level = node.address.depth() as usize;
        if self.levels.len() <= level {
            return;
        }
        self.levels[level]
            .address_node_map
            .insert(node.address, node);
    }

    pub fn get_octree_depth(node_shape: &ndshape::RuntimeShape<u32, 3>) -> u8 {
        let node_shape_size = node_shape.as_array();
        (node_shape_size[0] as f32).log2().ceil() as u8
    }

    /// size: is the size of the sampler_data
    #[allow(clippy::too_many_arguments)]
    pub fn build_bottom_up<S>(
        octree: &mut Octree,
        sampler_data: &[f32],
        shape: &S,
        voxel_size: f32,
        qef_stddev: f32,
        octree_offset: Vec3A,
        sampler_source: &impl OctreeSampler,
    ) where
        S: ndshape::Shape<3, Coord = u32>,
    {
        let _span = info_span!("octree build",);

        debug!(
            "octree_offset: {}, voxel size: {}, shape size: {}",
            octree_offset,
            voxel_size,
            shape.size()
        );

        octree.levels.resize_with(
            (Octree::get_octree_depth(&octree.node_shape) + 1) as usize,
            OctreeLevel::default,
        );

        Octree::build_leaf_nodes(
            shape,
            octree,
            sampler_data,
            voxel_size,
            octree_offset,
            sampler_source,
            qef_stddev,
        );

        Octree::build_bottom_up_from_leaf_nodes(octree, voxel_size);
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(fields(shape_size = shape.size()), skip(shape, octree, sampler_data, sampler_source))]
    fn build_leaf_nodes<S>(
        shape: &S,
        octree: &mut Octree,
        sampler_data: &[f32],
        voxel_size: f32,
        octree_offset: Vec3A,
        sampler_source: &impl OctreeSampler,
        qef_stddev: f32,
    ) where
        S: ndshape::Shape<3, Coord = u32>,
    {
        let size = shape.as_array();
        assert_eq!(size[0], size[1]);
        assert_eq!(size[0], size[2]);
        assert_eq!(
            ((size[0]) as f32).log2().ceil(),
            ((size[0] - 1) as f32).log2().ceil() + 1.0
        );

        let node_shape_size = octree.node_shape.as_array();

        let depth = Octree::get_octree_depth(&octree.node_shape);
        let address_node_map = &mut octree.levels[depth as usize].address_node_map;

        // build children leaf node
        for x in 0..node_shape_size[0] {
            for y in 0..node_shape_size[1] {
                for z in 0..node_shape_size[2] {
                    let mut conner_sampler_data = [0.0; 8];

                    for i in VertexIndex::iter() {
                        let offset = i.to_array();
                        let index = shape.linearize([
                            x + offset[0] as u32,
                            y + offset[1] as u32,
                            z + offset[2] as u32,
                        ]);
                        conner_sampler_data[i as usize] = sampler_data[index as usize];
                    }

                    if conner_sampler_data.iter().all(|v| *v < 0.0) {
                        continue;
                    }

                    if conner_sampler_data.iter().all(|v| *v >= 0.0) {
                        continue;
                    }

                    let morton_code = MortonCode::encode(UVec3::new(x, y, z), depth);

                    trace!(
                        "leaf coord:{}, {}, {}, conner_sampler_data: {:?}, address: {:?}",
                        x,
                        y,
                        z,
                        conner_sampler_data,
                        morton_code
                    );

                    let mut node = Node::new(NodeType::Leaf, morton_code);

                    let node_size = voxel_size;
                    let node_half_size = voxel_size * 0.5;
                    node.aabb = Aabb3d::new(
                        Vec3A::new(
                            x as f32 * node_size + node_half_size,
                            y as f32 * node_size + node_half_size,
                            z as f32 * node_size + node_half_size,
                        ) + octree_offset,
                        Vec3A::splat(node_half_size),
                    );
                    node.estimate_vertex(sampler_source, conner_sampler_data, qef_stddev);

                    address_node_map.insert(morton_code, node);
                }
            }
        }

        debug!("leaf octree.node_addresses len: {}", address_node_map.len());
    }

    #[instrument(skip(octree))]
    pub fn build_bottom_up_from_leaf_nodes(octree: &mut Octree, voxel_size: f32) {
        let mut node_shape_size = octree.node_shape.as_array();
        node_shape_size = [
            node_shape_size[0] / 2,
            node_shape_size[1] / 2,
            node_shape_size[2] / 2,
        ];
        let mut node_shape = ndshape::RuntimeShape::<u32, 3>::new([
            node_shape_size[0],
            node_shape_size[1],
            node_shape_size[2],
        ]);

        let depth = Octree::get_octree_depth(&octree.node_shape);
        for i in (0..depth).rev() {
            debug!("depth: {}", i);
            let mut current_address_node_map =
                std::mem::take(&mut octree.levels[i as usize].address_node_map);
            let last_address_node_map = &octree.levels[(i + 1) as usize].address_node_map;
            for (last_morton_code, last_node) in last_address_node_map.iter() {
                let morton_code = last_morton_code.parent().unwrap();

                let mut exist_child = false;
                let mut all_children_leaf = true;
                let mut mid_mat = None;
                let mut node_mats = [None; VertexIndex::COUNT];
                let mut conner_values = [None; 8];
                for (children_index, child_address) in
                    morton_code.children().unwrap().iter().enumerate()
                {
                    let child_node = last_address_node_map.get(child_address);
                    if let Some(child_node) = child_node {
                        exist_child = true;
                        match child_node.node_type {
                            NodeType::Branch => {
                                all_children_leaf = false;
                            }
                            NodeType::Leaf => {
                                // child node的children_index对角的点的材质
                                mid_mat = Some(child_node.vertices_mat_types[7 - children_index]);
                                node_mats[children_index] =
                                    Some(child_node.vertices_mat_types[children_index]);
                                conner_values[children_index] =
                                    Some(child_node.conner_sampler_data[children_index]);
                            }
                        }
                    }
                }

                let node_size = voxel_size * 2.0f32.powi((depth - i) as i32);
                let node_half_size = node_size * 0.5;
                if exist_child {
                    let mut node = Node::new(NodeType::Branch, morton_code);
                    let last_subnode_index = last_morton_code.get_subnode_index();
                    node.aabb = OctreeUtil::get_parent_node_aabb(
                        last_node.aabb,
                        last_subnode_index.to_index() as u8,
                    );
                    trace!("depth: {}, aabb half size: {}", i, node_half_size);
                    if all_children_leaf {
                        for (vertex_index, _) in VertexIndex::iter().enumerate() {
                            match node_mats[vertex_index] {
                                Some(mat) => node.vertices_mat_types[vertex_index] = mat,
                                None => node.vertices_mat_types[vertex_index] = mid_mat.unwrap(),
                            }
                        }
                    }
                    current_address_node_map.insert(morton_code, node);
                }
            }

            debug!(
                "depth {}, node_addresses len: {}",
                i,
                current_address_node_map.len()
            );

            octree.levels[i as usize].address_node_map = current_address_node_map;

            node_shape_size = node_shape.as_array();
            node_shape_size = [
                node_shape_size[0] / 2,
                node_shape_size[1] / 2,
                node_shape_size[2] / 2,
            ];
            node_shape = ndshape::RuntimeShape::<u32, 3>::new([
                node_shape_size[0],
                node_shape_size[1],
                node_shape_size[2],
            ]);
        }
    }

    #[instrument(skip_all, fields(shape_size = node_shape.size()))]
    pub fn simplify_octree(
        address_node_map: Arc<RwLock<HashMap<MortonCode, Node>>>,
        node_shape: ndshape::RuntimeShape<u32, 3>,
        qef_threshold_map: HashMap<u8, f32>,
    ) {
        let mut address_node_map = address_node_map.write().unwrap();

        let depth = Octree::get_octree_depth(&node_shape);

        let mut node_shape_size = node_shape.as_array();
        node_shape_size = [
            node_shape_size[0] / 2,
            node_shape_size[1] / 2,
            node_shape_size[2] / 2,
        ];
        let mut node_shape = ndshape::RuntimeShape::<u32, 3>::new([
            node_shape_size[0],
            node_shape_size[1],
            node_shape_size[2],
        ]);

        for i in (0..depth).rev() {
            trace!("simplify_octree depth: {}", i);
            let qef_threshold = qef_threshold_map.get(&i).unwrap_or(&0.1);

            // let last_address_node_map = &octree.levels[(i + 1) as usize].address_node_map;

            for x in 0..node_shape_size[0] {
                for y in 0..node_shape_size[1] {
                    for z in 0..node_shape_size[2] {
                        let morton_code = MortonCode::encode(UVec3::new(x, y, z), i);

                        if address_node_map.get_mut(&morton_code).is_none() {
                            continue;
                        }

                        let mut all_children_leaf = true;
                        let mut leaf_children_count = 0;
                        let mut qef = Quadric::default();
                        let mut avg_normal = Vec3::ZERO;
                        let mut avg_position = Vec3::ZERO;

                        for child_address in morton_code.children().unwrap().iter() {
                            let child_node = address_node_map.get(child_address);
                            if let Some(child_node) = child_node {
                                match child_node.node_type {
                                    NodeType::Branch => {
                                        all_children_leaf = false;
                                    }
                                    NodeType::Leaf => {
                                        if let Some(child_qef) = child_node.qef {
                                            qef += child_qef;
                                            avg_normal += child_node.normal_estimate;
                                            // 应该没用，因为有误差限制，不应该允许超出Node的误差限制。
                                            avg_position += child_node.vertex_estimate;
                                            leaf_children_count += 1;
                                        }
                                    }
                                }
                            }
                        }

                        let Some(node) = address_node_map.get_mut(&morton_code) else {
                            continue;
                        };

                        if all_children_leaf {
                            avg_normal /= leaf_children_count as f32;
                            avg_position /= leaf_children_count as f32;
                            node.estimate_vertex_with_qef(
                                qef,
                                avg_position.into(),
                                avg_normal.normalize().into(),
                            );
                            trace!("node, vertex: {}, qef error {}, address: {:?}, leaf_children_count: {}", node.vertex_estimate, node.qef_error, node.address, leaf_children_count);
                            if node.qef_error < *qef_threshold
                                && node.aabb.closest_point(node.vertex_estimate)
                                    == node.vertex_estimate.into()
                            {
                                node.node_type = NodeType::Leaf;

                                for child_address in morton_code.children().unwrap() {
                                    address_node_map.remove(&child_address);
                                }
                            }
                        }
                    }
                }
            }

            node_shape_size = node_shape.as_array();
            node_shape_size = [
                node_shape_size[0] / 2,
                node_shape_size[1] / 2,
                node_shape_size[2] / 2,
            ];
            node_shape = ndshape::RuntimeShape::<u32, 3>::new([
                node_shape_size[0],
                node_shape_size[1],
                node_shape_size[2],
            ]);

            debug!(
                "depth {}, node_addresses len: {}",
                i,
                address_node_map.len()
            );
        }
    }
}

macro_rules! check_octree_nodes_relation {
    ($octree: expr) => {
        #[cfg(debug_assertions)]
        Octree::check_nodes_relation($octree);
    };
}
pub(crate) use check_octree_nodes_relation;

impl Octree {
    #[cfg(debug_assertions)]
    pub fn check_nodes_relation(octree: &Octree) {
        for level in octree.levels.iter() {
            level
                .address_node_map
                .iter()
                .for_each(|(address, node)| match node.node_type {
                    NodeType::Branch => {
                        assert_eq!(node.address, *address);
                        let mut exist_child = false;
                        for child_address in node.address.children().unwrap() {
                            if octree.get_node(&child_address).is_some() {
                                exist_child = true;
                                break;
                            }
                        }
                        assert!(exist_child);
                    }
                    NodeType::Leaf => {
                        assert_eq!(node.address, *address);
                        for child_address in node.address.children().unwrap() {
                            assert!(octree.get_node(&child_address).is_none());
                        }
                    }
                });
        }
    }

    //      (2)o--------------o(3)
    //        /.             /|
    //       / .            / |
    //      /  .           /  |
    //     /   .          /   |     ^ Y
    // (6)o--------------o(7) |     |
    //    | (0)o. . . . . |. . o(1)  --> X
    //    |   .          |   /     /
    //    |  .           |  /     /
    //    | .            | /     z
    //    |.             |/
    // (4)o--------------o(5)
    //
    const SELECT_SEAM_LEAF_NODE_BY_AABB_FN: [fn(
        voxel_aabb: &Aabb3d,
        current_chunk_aabb: &Aabb3d,
    ) -> bool; SubNodeIndex::COUNT] = [
        |voxel_aabb: &Aabb3d, current_chunk_aabb: &Aabb3d| -> bool {
            voxel_aabb.max.x == current_chunk_aabb.max.x
                || voxel_aabb.max.y == current_chunk_aabb.max.y
                || voxel_aabb.max.z == current_chunk_aabb.max.z
        },
        |voxel_aabb: &Aabb3d, current_chunk_aabb: &Aabb3d| -> bool {
            voxel_aabb.min.x == current_chunk_aabb.max.x
                && (current_chunk_aabb.min.y <= voxel_aabb.min.y
                    && voxel_aabb.max.y <= current_chunk_aabb.max.y)
                && (current_chunk_aabb.min.z <= voxel_aabb.min.z
                    && voxel_aabb.max.z <= current_chunk_aabb.max.z)
        },
        |voxel_aabb: &Aabb3d, current_chunk_aabb: &Aabb3d| -> bool {
            voxel_aabb.min.y == current_chunk_aabb.max.y
                && (current_chunk_aabb.min.x <= voxel_aabb.min.x
                    && voxel_aabb.max.x <= current_chunk_aabb.max.x)
                && (current_chunk_aabb.min.z <= voxel_aabb.min.z
                    && voxel_aabb.max.z <= current_chunk_aabb.max.z)
        },
        |voxel_aabb: &Aabb3d, current_chunk_aabb: &Aabb3d| -> bool {
            voxel_aabb.min.y == current_chunk_aabb.max.y
                && voxel_aabb.min.x == current_chunk_aabb.max.x
                && (current_chunk_aabb.min.z <= voxel_aabb.min.z
                    && voxel_aabb.max.z <= current_chunk_aabb.max.z)
        },
        |voxel_aabb: &Aabb3d, current_chunk_aabb: &Aabb3d| -> bool {
            voxel_aabb.min.z == current_chunk_aabb.max.z
                && (current_chunk_aabb.min.x <= voxel_aabb.min.x
                    && voxel_aabb.max.x <= current_chunk_aabb.max.x)
                && (current_chunk_aabb.min.y <= voxel_aabb.min.y
                    && voxel_aabb.max.y <= current_chunk_aabb.max.y)
        },
        |voxel_aabb: &Aabb3d, current_chunk_aabb: &Aabb3d| -> bool {
            voxel_aabb.min.x == current_chunk_aabb.max.x
                && voxel_aabb.min.z == current_chunk_aabb.max.z
                && (current_chunk_aabb.min.y <= voxel_aabb.min.y
                    && voxel_aabb.max.y <= current_chunk_aabb.max.y)
        },
        |voxel_aabb: &Aabb3d, current_chunk_aabb: &Aabb3d| -> bool {
            voxel_aabb.min.y == current_chunk_aabb.max.y
                && voxel_aabb.min.z == current_chunk_aabb.max.z
                && (current_chunk_aabb.min.x <= voxel_aabb.min.x
                    && voxel_aabb.max.x <= current_chunk_aabb.max.x)
        },
        |voxel_aabb: &Aabb3d, current_chunk_aabb: &Aabb3d| -> bool {
            voxel_aabb.min.x == current_chunk_aabb.max.x
                && voxel_aabb.min.y == current_chunk_aabb.max.y
                && voxel_aabb.min.z == current_chunk_aabb.max.z
        },
    ];

    // 得到对应node的所有的seam的leaf node
    pub fn get_all_seam_leaf_nodes_by_aabb(
        chunk_border_vertices: &TerrainChunkBorderVertices,
        chunk_aabb: Aabb3d,
        subnode_index: SubNodeIndex,
    ) -> Vec<usize> {
        chunk_border_vertices
            .vertices_aabb
            .iter()
            .enumerate()
            .filter_map(|(index, voxel_aabb)| {
                if Octree::SELECT_SEAM_LEAF_NODE_BY_AABB_FN[subnode_index as usize](
                    voxel_aabb,
                    &chunk_aabb,
                ) {
                    trace!(
                        "get_all_seam_leaf_nodes, success: node aabb: {:?}, octree aabb: {:?}",
                        voxel_aabb,
                        chunk_aabb
                    );
                    Some(index)
                } else {
                    trace!(
                        "get_all_seam_leaf_nodes, fail: node aabb: {:?}, octree aabb: {:?}",
                        voxel_aabb,
                        chunk_aabb
                    );
                    None
                }
            })
            .collect::<Vec<usize>>()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_pow() {
        let a = 2.0f32.powi(3);
        assert_eq!(a, 8.0);

        let a = 2.0f32.powi(0);
        assert_eq!(a, 1.0);

        let a = 2.0f32.powi(-1);
        assert_eq!(a, 0.5);

        let a = 2.0f32.powi(-3);
        assert_eq!(a, 0.125);
    }

    #[test]
    fn test_rev_iter() {
        let depth = 3;
        for i in (0..depth).rev() {
            println!("i: {}", i);
        }
    }
}
