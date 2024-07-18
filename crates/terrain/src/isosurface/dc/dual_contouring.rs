use bevy::{
    math::{bounding::BoundingVolume, Vec3, Vec3A},
    utils::HashMap,
};
use strum::{EnumCount, IntoEnumIterator};
use tracing::{instrument, trace};

use crate::isosurface::{
    dc::octree::tables::{EDGE_NODES_VERTICES, FACE_TO_SUB_EDGES_AXIS_TYPE},
    surface::shape_surface::ShapeSurface,
};

use super::octree::{
    self,
    address::NodeAddress,
    node::{Node, NodeType},
    tables::{
        AxisType, SubNodeIndex, FACES_SUBNODES_NEIGHBOUR_PAIRS, FACES_TO_SUB_EDGES_NODES,
        SUBEDGE_NODES, SUBNODE_EDGES_NEIGHBOUR_PAIRS, SUBNODE_FACES_NEIGHBOUR_PAIRS,
    },
    OctreeProxy,
};

pub struct DefaultDualContouringVisiter<'a, 'b> {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3A>,
    pub tri_indices: Vec<u32>,
    pub address_vertex_id_map: HashMap<NodeAddress, u32>,
    pub shape_surface: &'a std::sync::RwLockReadGuard<'b, ShapeSurface>,
}

impl<'a, 'b> DefaultDualContouringVisiter<'a, 'b> {
    pub fn new(shape_surface: &'a std::sync::RwLockReadGuard<'b, ShapeSurface>) -> Self {
        Self {
            shape_surface,
            positions: Default::default(),
            normals: Default::default(),
            address_vertex_id_map: Default::default(),
            tri_indices: Default::default(),
        }
    }
}

impl<'a, 'b> DualContouringVisiter for DefaultDualContouringVisiter<'a, 'b> {
    fn visit_node(&mut self, node: &Node) {
        let old = self
            .address_vertex_id_map
            .insert(node.address, self.positions.len() as u32);
        assert!(old.is_none(), "node address is duplicated!");
        self.positions.push(node.vertex_estimate);
        self.normals.push(node.normal_estimate);
    }

    fn visit_triangle(&mut self, nodes: [&octree::node::Node; 3]) {
        let vertex_0 = self.address_vertex_id_map.get(&nodes[0].address).unwrap();
        let vertex_1 = self.address_vertex_id_map.get(&nodes[1].address).unwrap();
        let vertex_2 = self.address_vertex_id_map.get(&nodes[2].address).unwrap();
        self.tri_indices
            .extend_from_slice(&[*vertex_0, *vertex_1, *vertex_2]);
    }

    fn visit_quad(&mut self, nodes: [&octree::node::Node; 4]) {
        let vertex_0 = self.address_vertex_id_map.get(&nodes[0].address).unwrap();
        let vertex_1 = self.address_vertex_id_map.get(&nodes[1].address).unwrap();
        let vertex_2 = self.address_vertex_id_map.get(&nodes[2].address).unwrap();
        let vertex_3 = self.address_vertex_id_map.get(&nodes[3].address).unwrap();
        self.tri_indices
            .extend_from_slice(&[*vertex_0, *vertex_2, *vertex_1]);
        self.tri_indices
            .extend_from_slice(&[*vertex_1, *vertex_2, *vertex_3]);
    }
}

pub trait DualContouringVisiter {
    fn visit_node(&mut self, node: &Node);

    fn visit_triangle(&mut self, nodes: [&Node; 3]);

    fn visit_quad(&mut self, nodes: [&Node; 4]);
}

#[instrument(skip_all)]
pub fn dual_contouring(octree: &OctreeProxy, visiter: &mut impl DualContouringVisiter) {
    let root_address = NodeAddress::root();
    if let Some(root_node) = octree.node_addresses.get(&root_address) {
        node_proc(octree, root_node, visiter);
    }
}

struct FaceNodes<'a> {
    pub nodes: [&'a Node; 2],
    pub axis_type: AxisType,
}

struct EdgeNodes<'a> {
    // node的存储顺序是，沿着axis的正方向看去，
    // 2 3
    // 0 1
    // 也就是第0个node是左下角的node,
    // 也就是第1个node是右下角角的node,
    // 也就是第2个node是左上角的node,
    // 也就是第3个node是右上角的node,
    pub nodes: [&'a Node; 4],
    pub axis_type: AxisType,
    pub is_dup: [bool; 4],
}

#[instrument(skip_all)]
fn node_proc(octree: &OctreeProxy, parent_node: &Node, visiter: &mut impl DualContouringVisiter) {
    trace!("node proc: {:?}", parent_node.address);

    if parent_node.node_type == NodeType::Leaf {
        visiter.visit_node(parent_node);
        return;
    }

    // 8 subnode in one node
    let mut subnodes = [None; SubNodeIndex::COUNT];
    for subnode_index in SubNodeIndex::iter() {
        let node_address = parent_node.address.get_child_address(subnode_index);
        if let Some(subnode) = octree.node_addresses.get(&node_address) {
            node_proc(octree, subnode, visiter);
            subnodes[subnode_index as usize] = Some(subnode);
        }
    }

    // 12 faces within subnode in one node
    // 作用在于连接八叉树划分下的Node. 不断细分面，并通过边，将不同Node内部的点连接起来。
    (0..AxisType::COUNT).for_each(|axis| {
        for (left_node_index, right_node_index) in SUBNODE_FACES_NEIGHBOUR_PAIRS[axis] {
            if let (Some(left), Some(right)) = (
                subnodes[left_node_index as usize],
                subnodes[right_node_index as usize],
            ) {
                let face_nodes = FaceNodes {
                    nodes: [left, right],
                    axis_type: AxisType::from_repr(axis).unwrap(),
                };

                face_proc(octree, &face_nodes, visiter);
            }
        }
    });

    // 6 edges in one node
    // 连接八叉树划分下的Node. 且不断细分边，将不同Node内部的点连接起来。
    (0..AxisType::COUNT).for_each(|axis| {
        for (node_index_0, node_index_1, node_index_2, node_index_3) in
            SUBNODE_EDGES_NEIGHBOUR_PAIRS[axis]
        {
            if let (Some(node_1), Some(node_2), Some(node_3), Some(node_4)) = (
                subnodes[node_index_0 as usize],
                subnodes[node_index_1 as usize],
                subnodes[node_index_2 as usize],
                subnodes[node_index_3 as usize],
            ) {
                let nodes = [node_1, node_2, node_3, node_4];
                edge_proc(
                    octree,
                    EdgeNodes {
                        nodes,
                        axis_type: AxisType::from_repr(axis).unwrap(),
                        is_dup: [false; 4],
                    },
                    visiter,
                );
            }
        }
    });
}

/// get child node or self
fn get_child_node<'a>(
    octree: &'a OctreeProxy,
    node: &'a Node,
    subnode_index: SubNodeIndex,
) -> Option<&'a Node> {
    let node_address = node.address.get_child_address(subnode_index);
    match node.node_type {
        NodeType::Branch => octree.node_addresses.get(&node_address),
        NodeType::Leaf => Some(node),
    }
}

#[instrument(skip_all)]
fn face_proc(
    octree: &OctreeProxy,
    face_nodes: &FaceNodes,
    visiter: &mut impl DualContouringVisiter,
) {
    trace!(
        "face proc: axix type: {:?}, left node: {}, right node: {}",
        face_nodes.axis_type,
        face_nodes.nodes[0].address,
        face_nodes.nodes[1].address
    );
    // face proc
    // 4 faces in one face
    match (face_nodes.nodes[0].node_type, face_nodes.nodes[1].node_type) {
        (NodeType::Branch, NodeType::Branch)
        | (NodeType::Branch, NodeType::Leaf)
        | (NodeType::Leaf, NodeType::Branch) => {
            let subnode_indices: [(octree::tables::VertexIndex, octree::tables::VertexIndex); 4] =
                FACES_SUBNODES_NEIGHBOUR_PAIRS[face_nodes.axis_type as usize];
            for (subnode_index_0, subnode_index_1) in subnode_indices {
                let subnode_0 = get_child_node(octree, face_nodes.nodes[0], subnode_index_0);
                let subnode_1 = get_child_node(octree, face_nodes.nodes[1], subnode_index_1);

                if let (Some(subnode_0), Some(subnode_1)) = (subnode_0, subnode_1) {
                    let child_face_nodes = FaceNodes {
                        nodes: [subnode_0, subnode_1],
                        axis_type: face_nodes.axis_type,
                    };
                    face_proc(octree, &child_face_nodes, visiter);
                }
            }
        }
        (NodeType::Leaf, NodeType::Leaf) => {
            // 也没有边
            return;
        }
    }

    // 4 edges in one face
    for edge_axis_index in 0..2 {
        for edge_axis_value in 0..2 {
            let [(node_index_0, axis_value_0), (node_index_1, axis_value_1), (node_index_2, axis_value_2), (node_index_3, axis_value_3)] =
                FACES_TO_SUB_EDGES_NODES[face_nodes.axis_type as usize][edge_axis_index]
                    [edge_axis_value];

            let child_node_0 = get_child_node(
                octree,
                face_nodes.nodes[axis_value_0.bits() as usize],
                node_index_0,
            );
            let child_node_1 = get_child_node(
                octree,
                face_nodes.nodes[axis_value_1.bits() as usize],
                node_index_1,
            );
            let child_node_2 = get_child_node(
                octree,
                face_nodes.nodes[axis_value_2.bits() as usize],
                node_index_2,
            );
            let child_node_3 = get_child_node(
                octree,
                face_nodes.nodes[axis_value_3.bits() as usize],
                node_index_3,
            );
            if let (
                Some(child_node_0),
                Some(child_node_1),
                Some(child_node_2),
                Some(child_node_3),
            ) = (child_node_0, child_node_1, child_node_2, child_node_3)
            {
                let nodes = [child_node_0, child_node_1, child_node_2, child_node_3];
                // let is_dup = [
                //     nodes[0].address == face_nodes.nodes[axis_value_0.bits() as usize].address,
                //     nodes[1].address == face_nodes.nodes[axis_value_1.bits() as usize].address,
                //     nodes[2].address == face_nodes.nodes[axis_value_2.bits() as usize].address,
                //     nodes[3].address == face_nodes.nodes[axis_value_3.bits() as usize].address,
                // ];
                let edge_axis_type =
                    FACE_TO_SUB_EDGES_AXIS_TYPE[face_nodes.axis_type as usize][edge_axis_index];
                edge_proc(
                    octree,
                    EdgeNodes {
                        nodes,
                        axis_type: edge_axis_type,
                        is_dup: [false; 4],
                    },
                    visiter,
                );
            }
        }
    }
}

#[instrument(skip_all)]
fn edge_proc(
    octree: &OctreeProxy,
    edge_nodes: EdgeNodes,
    visiter: &mut impl DualContouringVisiter,
) {
    trace!(
        "edge proc: axix type: {:?}, 0 node: {}, 1 node: {}, 2 node: {}, 3 node: {}",
        edge_nodes.axis_type,
        edge_nodes.nodes[0].coord,
        edge_nodes.nodes[1].coord,
        edge_nodes.nodes[2].coord,
        edge_nodes.nodes[3].coord,
    );

    if edge_nodes
        .nodes
        .iter()
        .all(|node| node.node_type == NodeType::Leaf)
    {
        visit_leaf_edge(octree, edge_nodes, visiter);
        return;
    }

    // get sub edge nodes

    for i in 0..2 {
        let [node_1, node_2, node_3, node_4] = edge_nodes.nodes;
        let [subnode_index_1, subnode_index_2, subnode_index_3, subnode_index_4] =
            SUBEDGE_NODES[edge_nodes.axis_type as usize][i];

        let child_node_0 = get_child_node(octree, node_1, subnode_index_1);
        let child_node_1 = get_child_node(octree, node_2, subnode_index_2);
        let child_node_2 = get_child_node(octree, node_3, subnode_index_3);
        let child_node_3 = get_child_node(octree, node_4, subnode_index_4);

        trace!(
            "edge proc: {:?}, {:?}, {:?}, {:?}",
            child_node_0,
            child_node_1,
            child_node_2,
            child_node_3
        );
        if let (Some(child_node_0), Some(child_node_1), Some(child_node_2), Some(child_node_3)) =
            (child_node_0, child_node_1, child_node_2, child_node_3)
        {
            let sub_edge_nodes = [child_node_0, child_node_1, child_node_2, child_node_3];

            edge_proc(
                octree,
                EdgeNodes {
                    nodes: sub_edge_nodes,
                    axis_type: edge_nodes.axis_type,
                    is_dup: [
                    // sub_edge_nodes[0].address == node_1.address,
                    // sub_edge_nodes[1].address == node_2.address,
                    // sub_edge_nodes[2].address == node_3.address,
                    // sub_edge_nodes[3].address == node_4.address,
                    false; 4
                ],
                },
                visiter,
            );
        }
    }
}

#[instrument(skip_all)]
fn visit_leaf_edge(
    _octree: &OctreeProxy,
    edge_nodes: EdgeNodes,
    visiter: &mut impl DualContouringVisiter,
) {
    assert!(edge_nodes
        .nodes
        .iter()
        .all(|node| node.node_type == NodeType::Leaf));

    trace!(
        "leaf edge proc: axix type: {:?}, 0 node: {}, 1 node: {}, 2 node: {}, 3 node: {}",
        edge_nodes.axis_type,
        edge_nodes.nodes[0].coord,
        edge_nodes.nodes[1].coord,
        edge_nodes.nodes[2].coord,
        edge_nodes.nodes[3].coord
    );

    // Check if this leaf edge is bipolar. We can just check the samples on
    // the smallest node.
    let mut min_node_index = 0;
    let mut min_size = f32::MAX;
    for (i, node) in edge_nodes.nodes.iter().enumerate() {
        let half_size = node.aabb.half_size().x;
        if half_size < min_size {
            min_size = half_size;
            min_node_index = i;
        }
    }
    // Select the edge at the opposite corner of the octant.
    let node_vertex_indices = EDGE_NODES_VERTICES[edge_nodes.axis_type as usize][min_node_index];
    let vertex_mats = &edge_nodes.nodes[min_node_index].vertices_mat_types;
    let mat0 = vertex_mats[node_vertex_indices[0] as usize];
    let mat1 = vertex_mats[node_vertex_indices[1] as usize];

    let flip = match (mat0, mat1) {
        (octree::node::VoxelMaterialType::Air, octree::node::VoxelMaterialType::Block) => true,
        (octree::node::VoxelMaterialType::Block, octree::node::VoxelMaterialType::Air) => false,
        (octree::node::VoxelMaterialType::Block, octree::node::VoxelMaterialType::Block)
        | (octree::node::VoxelMaterialType::Air, octree::node::VoxelMaterialType::Air) => {
            // Not a bipolar edge.
            trace!(
                "visit leaf edge is not a bipolar edge, mat0: {:?}, mat1: {:?}, axis:{:?}, \
                min_node_index:{}, vertex_samplers:{:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                mat0,
                mat1,
                edge_nodes.axis_type,
                min_node_index,
                edge_nodes.nodes[0].coord,
                edge_nodes.nodes[1].coord,
                edge_nodes.nodes[2].coord,
                edge_nodes.nodes[3].coord,
                node_vertex_indices[0],
                node_vertex_indices[1],
            );
            return;
        }
    };
    trace!(
        "node pos: {:?}, {:?}, {:?}, {:?}, {:?}",
        edge_nodes.axis_type,
        edge_nodes.nodes[0].coord,
        edge_nodes.nodes[1].coord,
        edge_nodes.nodes[2].coord,
        edge_nodes.nodes[3].coord
    );

    // Filter triangles with duplicate vertices (from edges with duplicate
    // nodes). Because the triangles must share a diagonal, we know a
    // duplicate can't occur in both triangles. We also know that if any
    // duplicate exists, it will necessarily appear twice around this edge.
    let tris = [[0, 2, 1], [1, 2, 3]];
    let first_tri_num_dups = tris[0]
        .iter()
        .map(|&t| edge_nodes.is_dup[t] as u8)
        .sum::<u8>();
    if first_tri_num_dups > 0 {
        // Skip the degenerate triangle.
        let use_tri = if first_tri_num_dups == 1 {
            tris[0]
        } else {
            tris[1]
        };
        if flip {
            let flipped_tri = [use_tri[0], use_tri[2], use_tri[1]];
            visiter.visit_triangle(flipped_tri.map(|i| edge_nodes.nodes[i]));
        } else {
            visiter.visit_triangle(use_tri.map(|i| edge_nodes.nodes[i]));
        }
    } else {
        // No degenerate triangles found.
        if flip {
            visiter.visit_quad([
                edge_nodes.nodes[2],
                edge_nodes.nodes[3],
                edge_nodes.nodes[0],
                edge_nodes.nodes[1],
            ]);
        } else {
            visiter.visit_quad(edge_nodes.nodes);
        }
    }
}
