pub mod address;
pub mod node;
pub mod tables;

use core::panic;
use std::sync::{Arc, RwLock, RwLockReadGuard};

use address::DepthCoordMap;
use bevy::{
    math::{bounding::Aabb3d, Vec3A},
    prelude::*,
    utils::{tracing::instrument, HashMap},
};

use ndshape::Shape;
use node::{Node, NodeType};
use pqef::Quadric;
use strum::{EnumCount, IntoEnumIterator};
use tables::{SubNodeIndex, VertexIndex};

use address::NodeAddress;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum OctreeSubdivisionResult {
    Subdivided,
    MaxDepth,
    NotSubdivided,
}

pub trait OctreeBranchPolicy {
    fn check_to_subdivision(&self, aabb: Aabb3d) -> OctreeSubdivisionResult;
}

pub trait OctreeSampler {
    fn sampler(&self, loc: Vec3) -> f32;
    fn sampler_split(&self, x: f32, y: f32, z: f32) -> f32;
}

pub trait OctreeContext: OctreeBranchPolicy + OctreeSampler {}

/// octree
/// 1. 细分octree，是否可以细分
/// 2. 计算叶子的qef。
/// 3. 反向进行收缩，节省内容空间。
///
/// 自顶向下，还是自底向上？
///
/// 也许可以通过不断插入顶点，来决定是否应该细分。()
/// 性能比每次细分来检测所有的顶点数据要好（层级细分是层级数*(每层涵盖的顶点数 = 所有顶点数）)，
#[derive(Component)]
pub struct Octree {
    pub address_node_map: Arc<RwLock<HashMap<NodeAddress, Node>>>,
    pub node_shape: ndshape::RuntimeShape<u32, 3>,
}

#[derive()]
pub struct OctreeProxy<'a> {
    pub node_addresses: RwLockReadGuard<'a, HashMap<NodeAddress, Node>>,
    pub is_seam: bool,
    pub surface: RwLockReadGuard<'a, ShapeSurface>,
}

impl Octree {
    pub fn new(node_shape: ndshape::RuntimeShape<u32, 3>) -> Self {
        Octree {
            address_node_map: Arc::new(RwLock::new(HashMap::default())),
            node_shape,
        }
    }

    pub fn get_octree_depth(node_shape: &ndshape::RuntimeShape<u32, 3>) -> OctreeDepthType {
        let node_shape_size = node_shape.as_array();
        (node_shape_size[0] as f32).log2().ceil() as OctreeDepthType
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
        node_address: Arc<RwLock<HashMap<OctreeDepthType, Vec<NodeAddress>>>>,
    ) where
        S: ndshape::Shape<3, Coord = u32>,
    {
        let _span = debug_span!("octree build",);

        debug!(
            "octree_offset: {}, voxel size: {}, shape size: {}",
            octree_offset,
            voxel_size,
            shape.size()
        );

        Octree::build_leaf_nodes(
            shape,
            &node_address,
            octree,
            sampler_data,
            voxel_size,
            octree_offset,
            sampler_source,
            qef_stddev,
        );

        Octree::build_bottom_up_from_leaf_nodes(octree, voxel_size, octree_offset, node_address);
    }

    #[allow(clippy::too_many_arguments)]
    #[instrument(fields(shape_size = shape.size()), skip(shape, node_address, octree, sampler_data, sampler_source))]
    fn build_leaf_nodes<S>(
        shape: &S,
        node_address: &Arc<RwLock<HashMap<OctreeDepthType, Vec<NodeAddress>>>>,
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

        let node_address_guard = node_address.read().unwrap();
        let node_shape_size = octree.node_shape.as_array();

        let depth = Octree::get_octree_depth(&octree.node_shape);
        let Some(leaf_address_mapper) = node_address_guard.get(&depth) else {
            panic!(
                "depth {} is invalid! and node_mapper size: {}, and keys:{:?}",
                depth,
                node_address_guard.len(),
                node_address_guard.keys()
            );
        };

        let mut address_node_map = octree.address_node_map.write().unwrap();

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

                    let node_index = octree.node_shape.linearize([x, y, z]);
                    let node_address = leaf_address_mapper[node_index as usize];

                    trace!(
                "leaf coord:{}, {}, {}, conner_sampler_data: {:?}, address: {:?}, node_index:{}",
                x, y, z, conner_sampler_data, node_address, node_index);

                    let mut node = Node::new(NodeType::Leaf, node_address);

                    let node_size = voxel_size;
                    let node_half_size = voxel_size * 0.5;
                    node.coord = Vec3A::new(x as f32, y as f32, z as f32);
                    node.aabb = Aabb3d::new(
                        Vec3A::new(
                            x as f32 * node_size + node_half_size,
                            y as f32 * node_size + node_half_size,
                            z as f32 * node_size + node_half_size,
                        ) + octree_offset,
                        Vec3A::splat(node_half_size),
                    );
                    node.estimate_vertex(sampler_source, conner_sampler_data, qef_stddev);

                    address_node_map.insert(node_address, node);
                }
            }
        }

        debug!("leaf octree.node_addresses len: {}", address_node_map.len());
    }

    #[instrument(skip(octree, node_address))]
    pub fn build_bottom_up_from_leaf_nodes(
        octree: &mut Octree,
        voxel_size: f32,
        octree_offset: Vec3A,
        node_address: Arc<RwLock<HashMap<OctreeDepthType, Vec<NodeAddress>>>>,
    ) {
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

        let node_address = node_address.read().unwrap();
        let mut address_node_map = octree.address_node_map.write().unwrap();

        let depth = Octree::get_octree_depth(&octree.node_shape);
        for i in (0..depth).rev() {
            debug!("depth: {}", i);
            let node_address_mapper = node_address.get(&i).unwrap();
            for x in 0..node_shape_size[0] {
                for y in 0..node_shape_size[1] {
                    for z in 0..node_shape_size[2] {
                        let node_index = node_shape.linearize([x, y, z]);
                        let node_address = node_address_mapper[node_index as usize];

                        let mut exist_child = false;
                        let mut all_children_leaf = true;
                        let mut mid_mat = None;
                        let mut node_mats = [None; VertexIndex::COUNT];
                        let mut conner_values = [None; 8];
                        for (children_index, child_address) in
                            node_address.get_children_addresses().iter().enumerate()
                        {
                            let child_node = address_node_map.get(child_address);
                            if let Some(child_node) = child_node {
                                exist_child = true;
                                match child_node.node_type {
                                    NodeType::Branch => {
                                        all_children_leaf = false;
                                    }
                                    NodeType::Leaf => {
                                        // child node的children_index对角的点的材质
                                        mid_mat =
                                            Some(child_node.vertices_mat_types[7 - children_index]);
                                        // Some(VoxelMaterialType::Air);
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
                            let mut node = Node::new(NodeType::Branch, node_address);
                            node.coord = Vec3A::new(x as f32, y as f32, z as f32);
                            node.aabb = Aabb3d::new(
                                Vec3A::new(
                                    x as f32 * node_size + node_half_size,
                                    y as f32 * node_size + node_half_size,
                                    z as f32 * node_size + node_half_size,
                                ) + octree_offset,
                                Vec3A::splat(node_half_size),
                            );
                            // TODO move mat to simplify_octree
                            trace!("depth: {}, aabb half size: {}", i, node_half_size);
                            if all_children_leaf {
                                for (vertex_index, _) in VertexIndex::iter().enumerate() {
                                    match node_mats[vertex_index] {
                                        Some(mat) => node.vertices_mat_types[vertex_index] = mat,
                                        None => {
                                            node.vertices_mat_types[vertex_index] = mid_mat.unwrap()
                                        }
                                    }
                                    // let mut values = [0.0; 8];
                                    // for (i, value) in conner_values.iter().enumerate() {
                                    //     match value {
                                    //         Some(value) => values[i] = *value,
                                    //         None => values[i] = f32::MAX,
                                    //     }
                                    // }
                                    // node.estimate_vertex_mat(values);
                                }
                            }
                            address_node_map.insert(node_address, node);
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

            info!(
                "depth {}, node_addresses len: {}",
                i,
                address_node_map.len()
            );
        }
    }

    #[instrument(skip_all, fields(shape_size = node_shape.size()))]
    pub fn simplify_octree(
        address_node_map: Arc<RwLock<HashMap<NodeAddress, Node>>>,
        node_shape: ndshape::RuntimeShape<u32, 3>,
        deep_coord_mapper: Arc<RwLock<DepthCoordMap>>,
        qef_threshold_map: HashMap<OctreeDepthType, f32>,
    ) {
        let mut address_node_map = address_node_map.write().unwrap();
        let deep_coord_mapper = deep_coord_mapper.read().unwrap();
        debug!("leaf octree.node_addresses len: {}", address_node_map.len());

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
            let coord_address_vec = deep_coord_mapper.get(&i).unwrap();
            let qef_threshold = qef_threshold_map.get(&i).unwrap_or(&0.1);
            for x in 0..node_shape_size[0] {
                for y in 0..node_shape_size[1] {
                    for z in 0..node_shape_size[2] {
                        let node_index = node_shape.linearize([x, y, z]);
                        let node_address = coord_address_vec[node_index as usize];

                        if address_node_map.get_mut(&node_address).is_none() {
                            continue;
                        }

                        let mut all_children_leaf = true;
                        let mut leaf_children_count = 0;
                        let mut qef = Quadric::default();
                        let mut avg_normal = Vec3A::ZERO;
                        let mut avg_position = Vec3::ZERO;

                        for child_address in node_address.get_children_addresses().iter() {
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

                        let Some(node) = address_node_map.get_mut(&node_address) else {
                            continue;
                        };

                        if all_children_leaf {
                            avg_normal /= leaf_children_count as f32;
                            avg_position /= leaf_children_count as f32;
                            node.estimate_vertex_with_qef(
                                qef,
                                avg_position.into(),
                                avg_normal.normalize(),
                            );
                            trace!("node, vertex: {}, qef error {}, {}, coord:{:?}, leaf_children_count: {}", node.vertex_estimate, node.qef_error, node.address, node.coord, leaf_children_count);
                            if node.qef_error < *qef_threshold
                                && node.aabb.closest_point(node.vertex_estimate)
                                    == node.vertex_estimate.into()
                            {
                                node.node_type = NodeType::Leaf;

                                for child_address in node_address.get_children_addresses() {
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
    ($node_addresses: expr) => {
        #[cfg(debug_assertions)]
        Octree::check_nodes_relation($node_addresses);
    };
}
pub(crate) use check_octree_nodes_relation;

use crate::{
    chunk_mgr::chunk::chunk_lod::OctreeDepthType, isosurface::surface::shape_surface::ShapeSurface,
};

impl Octree {
    #[cfg(debug_assertions)]
    pub fn check_nodes_relation(node_addresses: Arc<RwLock<HashMap<NodeAddress, Node>>>) {
        let node_addresses = node_addresses.read().unwrap();
        node_addresses
            .iter()
            .for_each(|(address, node)| match node.node_type {
                NodeType::Branch => {
                    assert_eq!(node.address, *address);
                    let mut exist_child = false;
                    for child_address in node.address.get_children_addresses() {
                        if node_addresses.get(&child_address).is_some() {
                            exist_child = true;
                            break;
                        }
                    }
                    assert!(exist_child);
                }
                NodeType::Leaf => {
                    assert_eq!(node.address, *address);
                    for child_address in node.address.get_children_addresses() {
                        assert!(node_addresses.get(&child_address).is_none());
                    }
                }
            });
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
    const SELECT_SEAM_LEAF_NODE_FN: [fn(node: &Node, aabb: &Aabb3d) -> bool; SubNodeIndex::COUNT] = [
        |node: &Node, aabb: &Aabb3d| -> bool {
            node.aabb.max.x == aabb.max.x
                || node.aabb.max.y == aabb.max.y
                || node.aabb.max.z == aabb.max.z
        },
        |node: &Node, aabb: &Aabb3d| -> bool {
            node.aabb.min.x == aabb.max.x
                && (aabb.min.y <= node.aabb.min.y && node.aabb.max.y <= aabb.max.y)
                && (aabb.min.z <= node.aabb.min.z && node.aabb.max.z <= aabb.max.z)
        },
        |node: &Node, aabb: &Aabb3d| -> bool {
            node.aabb.min.y == aabb.max.y
                && (aabb.min.x <= node.aabb.min.x && node.aabb.max.x <= aabb.max.x)
                && (aabb.min.z <= node.aabb.min.z && node.aabb.max.z <= aabb.max.z)
        },
        |node: &Node, aabb: &Aabb3d| -> bool {
            node.aabb.min.y == aabb.max.y
                && node.aabb.min.x == aabb.max.x
                && (aabb.min.z <= node.aabb.min.z && node.aabb.max.z <= aabb.max.z)
        },
        |node: &Node, aabb: &Aabb3d| -> bool {
            node.aabb.min.z == aabb.max.z
                && (aabb.min.x <= node.aabb.min.x && node.aabb.max.x <= aabb.max.x)
                && (aabb.min.y <= node.aabb.min.y && node.aabb.max.y <= aabb.max.y)
        },
        |node: &Node, aabb: &Aabb3d| -> bool {
            node.aabb.min.x == aabb.max.x
                && node.aabb.min.z == aabb.max.z
                && (aabb.min.y <= node.aabb.min.y && node.aabb.max.y <= aabb.max.y)
        },
        |node: &Node, aabb: &Aabb3d| -> bool {
            node.aabb.min.y == aabb.max.y
                && node.aabb.min.z == aabb.max.z
                && (aabb.min.x <= node.aabb.min.x && node.aabb.max.x <= aabb.max.x)
        },
        |node: &Node, aabb: &Aabb3d| -> bool {
            node.aabb.min.x == aabb.max.x
                && node.aabb.min.y == aabb.max.y
                && node.aabb.min.z == aabb.max.z
        },
    ];

    // 得到对应node的所有的seam的leaf node
    pub fn get_all_seam_leaf_nodes<'a>(
        node_addresses: &'a RwLockReadGuard<HashMap<NodeAddress, Node>>,
        octree_aabb: Aabb3d,
        subnode_index: SubNodeIndex,
    ) -> Vec<&'a Node> {
        node_addresses
            .iter()
            .filter_map(|(_, node)| match node.node_type {
                NodeType::Branch => None,
                NodeType::Leaf => {
                    if Octree::SELECT_SEAM_LEAF_NODE_FN[subnode_index as usize](node, &octree_aabb)
                    {
                        trace!(
                            "get_all_seam_leaf_nodes, success: coord:{}, node aabb: {:?}, octree aabb: {:?}",
                            node.coord,
                            node.aabb,
                            octree_aabb
                        );
                        Some(node)
                    } else {
                        trace!(
                            "get_all_seam_leaf_nodes, fail: coord:{}, node aabb: {:?}, octree aabb: {:?}",
                            node.coord,
                            node.aabb,
                            octree_aabb
                        );
                        None
                    }
                }
            })
            .collect::<Vec<&'a Node>>()
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
