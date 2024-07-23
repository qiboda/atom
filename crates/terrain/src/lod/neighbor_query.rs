use bevy::prelude::*;

use crate::{
    isosurface::dc::octree::{
        address::NodeAddress,
        tables::{
            EdgeIndex, FaceIndex, VertexIndex, TWIN_EDGE_INDEX, TWIN_FACE_INDEX, TWIN_VERTEX_INDEX,
        },
    },
    lod::lod_octree::LodOctreeNodeType,
};

use super::lod_octree::{LodOctreeMap, LodOctreeNode};

fn get_parent_lod_octree_node(
    query: &Query<&LodOctreeNode>,
    lod_octree_map: &Res<LodOctreeMap>,
    current_address: NodeAddress,
) -> NodeAddress {
    let parent_address = current_address.get_parent_address();
    let node_entity = lod_octree_map.get_node_entity(parent_address);
    match node_entity {
        Some(node_entity) => {
            let parent_node = query
                .get(*node_entity)
                .expect("can not found parent node, lod_octree_map and query should be sync");
            assert!(parent_node.node_type == LodOctreeNodeType::Leaf);
            parent_address
        }
        None => get_parent_lod_octree_node(query, lod_octree_map, parent_address),
    }
}

fn get_child_lod_octree_nodes_by_vertex(
    query: &Query<&LodOctreeNode>,
    lod_octree_map: &Res<LodOctreeMap>,
    parent_address: NodeAddress,
    vertex_index: VertexIndex,
) -> NodeAddress {
    let child_address = parent_address.get_children_addresses_by_vertex(vertex_index);
    let child_node_entity = lod_octree_map.get_node_entity(child_address);
    match child_node_entity {
            Some(child_node_entity) => {
                let child_node = query
                    .get(*child_node_entity)
                    .expect("can not found child node, lod_octree_map and query should sync");
                if child_node.node_type == LodOctreeNodeType::Leaf {
                    child_address
                } else {
                    get_child_lod_octree_nodes_by_vertex(
                        query,
                        lod_octree_map,
                        child_address,
                        vertex_index,
                    )
                }
            }
            None => panic!("lod octree always has full nodes. and use this function only when parent node is internal node"),
        }
}

fn get_children_lod_octree_nodes_by_edge(
    query: &Query<&LodOctreeNode>,
    lod_octree_map: &Res<LodOctreeMap>,
    parent_address: NodeAddress,
    edge_index: EdgeIndex,
) -> Vec<NodeAddress> {
    let mut children = vec![];
    for child_address in parent_address.get_children_addresses_by_edge(edge_index) {
        let child_node_entity = lod_octree_map.get_node_entity(child_address);
        match child_node_entity {
            Some(child_node_entity) => {
                let child_node = query
                    .get(*child_node_entity)
                    .expect("can not found child node, lod_octree_map and query should sync");
                if child_node.node_type == LodOctreeNodeType::Leaf {
                    children.push(child_address);
                } else {
                    children.extend(get_children_lod_octree_nodes_by_edge(
                        query,
                        lod_octree_map,
                        child_address,
                        edge_index,
                    ));
                }
            }
            None => panic!("lod octree always has full nodes. and use this function only when parent node is internal node"),
        }
    }
    children
}

fn get_children_lod_octree_nodes_by_face(
    query: &Query<&LodOctreeNode>,
    lod_octree_map: &Res<LodOctreeMap>,
    parent_address: NodeAddress,
    face_index: FaceIndex,
) -> Vec<NodeAddress> {
    let mut children = vec![];
    for child_address in parent_address.get_children_addresses_by_face(face_index) {
        let child_node_entity = lod_octree_map.get_node_entity(child_address);
        match child_node_entity {
            Some(child_node_entity) => {
                let child_node = query
                    .get(*child_node_entity)
                    .expect("can not found child node, lod_octree_map and query should sync");
                if child_node.node_type == LodOctreeNodeType::Leaf {
                    children.push(child_address);
                } else {
                    children.extend(get_children_lod_octree_nodes_by_face(
                        query,
                        lod_octree_map,
                        child_address,
                        face_index,
                    ));
                }
            }
            None => panic!("lod octree always has full nodes. and use this function only when parent node is internal node"),
        }
    }
    children
}

pub fn get_vertex_neighbor_lod_octree_nodes(
    query: &Query<&LodOctreeNode>,
    lod_octree_map: &Res<LodOctreeMap>,
    current_address: NodeAddress,
    vertex_index: VertexIndex,
) -> NodeAddress {
    let neighbor_address = current_address.get_vertex_neighbor_address(vertex_index);
    let neighbor_node_entity = lod_octree_map.get_node_entity(neighbor_address);
    match neighbor_node_entity {
        Some(entity) => {
            let neighbor_node = query
                .get(*entity)
                .expect("can not found neighbor node, lod_octree_map and query should sync");
            if neighbor_node.node_type == LodOctreeNodeType::Leaf {
                neighbor_address
            } else {
                get_child_lod_octree_nodes_by_vertex(
                    query,
                    lod_octree_map,
                    neighbor_address,
                    TWIN_VERTEX_INDEX[vertex_index as usize],
                )
            }
        }
        None => {
            // 发现父节点。
            get_parent_lod_octree_node(query, lod_octree_map, neighbor_address)
        }
    }
}

pub fn get_edge_neighbor_lod_octree_nodes(
    query: &Query<&LodOctreeNode>,
    lod_octree_map: &Res<LodOctreeMap>,
    current_address: NodeAddress,
    edge_index: EdgeIndex,
) -> Vec<NodeAddress> {
    let neighbor_address = current_address.get_edge_neighbor_address(edge_index);
    let neighbor_node_entity = lod_octree_map.get_node_entity(neighbor_address);
    match neighbor_node_entity {
        Some(entity) => {
            let neighbor_node = query
                .get(*entity)
                .expect("can not found neighbor node, lod_octree_map and query should sync");
            if neighbor_node.node_type == LodOctreeNodeType::Leaf {
                vec![neighbor_address]
            } else {
                get_children_lod_octree_nodes_by_edge(
                    query,
                    lod_octree_map,
                    neighbor_address,
                    TWIN_EDGE_INDEX[edge_index as usize],
                )
            }
        }
        None => {
            // 发现父节点。
            vec![get_parent_lod_octree_node(
                query,
                lod_octree_map,
                neighbor_address,
            )]
        }
    }
}

pub fn get_face_neighbor_lod_octree_nodes(
    query: &Query<&LodOctreeNode>,
    lod_octree_map: &Res<LodOctreeMap>,
    current_address: NodeAddress,
    face_index: FaceIndex,
) -> Vec<NodeAddress> {
    let neighbor_address = current_address.get_face_neighbor_address(face_index);

    let neighbor_node_entity = lod_octree_map.get_node_entity(neighbor_address);
    match neighbor_node_entity {
        Some(entity) => {
            let neighbor_node = query
                .get(*entity)
                .expect("can not found neighbor node, lod_octree_map and query should sync");
            if neighbor_node.node_type == LodOctreeNodeType::Leaf {
                vec![neighbor_address]
            } else {
                get_children_lod_octree_nodes_by_face(
                    query,
                    lod_octree_map,
                    neighbor_address,
                    TWIN_FACE_INDEX[face_index as usize],
                )
            }
        }
        None => {
            // 发现父节点。
            vec![get_parent_lod_octree_node(
                query,
                lod_octree_map,
                neighbor_address,
            )]
        }
    }
}
