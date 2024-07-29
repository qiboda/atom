use std::ops::Not;

use bevy::utils::hashbrown::HashSet;

use super::{
    lod_octree::{TerrainLodOctree, TerrainLodOctreeNode},
    morton_code::MortonCode,
    morton_code_neighbor::MortonCodeNeighbor,
};
use crate::tables::{
    EdgeIndex, FaceIndex, VertexIndex, TWIN_EDGE_INDEX, TWIN_FACE_INDEX, TWIN_VERTEX_INDEX,
};

fn get_parent_lod_octree_node<'a>(
    lod_octree_map: &'a TerrainLodOctree,
    current_node_code: &MortonCode,
    depth: u8,
) -> Option<&'a TerrainLodOctreeNode> {
    if depth == 0 {
        return None;
    }

    if let Some(parent_code) = current_node_code.parent() {
        return match lod_octree_map.get_node(&parent_code) {
            Some(node) => Some(node),
            None => get_parent_lod_octree_node(lod_octree_map, &parent_code, depth - 1),
        };
    }
    None
}

fn get_child_lod_octree_nodes_by_vertex(
    lod_octree_map: &TerrainLodOctree,
    parent_code: MortonCode,
    vertex_index: VertexIndex,
    depth: u8,
) -> Vec<&TerrainLodOctreeNode> {
    let mut children = vec![];
    if depth == 0 {
        return children;
    }
    let children_address = parent_code.get_children_morton_code_by_vertex(vertex_index);
    for child in children_address {
        let child_node = lod_octree_map.get_node(&child);
        match child_node {
            Some(child_node) => {
                children.push(child_node);
            }
            None => {
                children.extend(get_child_lod_octree_nodes_by_vertex(
                    lod_octree_map,
                    child,
                    vertex_index,
                    depth - 1,
                ));
            }
        }
    }
    children
}

fn get_children_lod_octree_nodes_by_edge(
    lod_octree_map: &TerrainLodOctree,
    parent_address: MortonCode,
    edge_index: EdgeIndex,
    depth: u8,
) -> Vec<&TerrainLodOctreeNode> {
    let mut children = vec![];
    if depth == 0 {
        return children;
    }
    for child_code in parent_address.get_children_morton_code_by_edge(edge_index) {
        let child_node = lod_octree_map.get_node(&child_code);
        match child_node {
            Some(child_node) => {
                children.push(child_node);
            }
            None => {
                children.extend(get_children_lod_octree_nodes_by_edge(
                    lod_octree_map,
                    child_code,
                    edge_index,
                    depth - 1,
                ));
            }
        }
    }
    children
}

fn get_children_lod_octree_nodes_by_face(
    lod_octree: &TerrainLodOctree,
    parent_code: MortonCode,
    face_index: FaceIndex,
    depth: u8,
) -> Vec<&TerrainLodOctreeNode> {
    let mut children = vec![];
    if depth == 0 {
        return children;
    }
    for child_address in parent_code.get_children_morton_code_by_face(face_index) {
        let node = lod_octree.get_node(&child_address);
        match node {
            Some(node) => {
                children.push(node);
            }
            None => {
                children.extend(
                    get_children_lod_octree_nodes_by_face(
                        lod_octree,
                        child_address,
                        face_index,
                        depth - 1,
                    )
                    .iter(),
                );
            }
        }
    }
    children
}

pub fn get_vertex_neighbor_lod_octree_nodes<'a>(
    lod_octree: &'a TerrainLodOctree,
    current_node: &TerrainLodOctreeNode,
    vertex_index: VertexIndex,
    depth: u8,
) -> Vec<&'a TerrainLodOctreeNode> {
    let mut nodes = vec![];
    let neighbor_code = current_node
        .code
        .get_neighbor_vertex_morton_code(vertex_index);
    if let Ok(neighbor_code) = neighbor_code {
        let neighbor_node = lod_octree.get_node(&neighbor_code);
        match neighbor_node {
            Some(node) => {
                nodes.push(node);
            }
            None => {
                let parent_node = get_parent_lod_octree_node(lod_octree, &neighbor_code, depth);
                match parent_node {
                    Some(node) => {
                        nodes.push(node);
                    }
                    None => nodes.extend(get_child_lod_octree_nodes_by_vertex(
                        lod_octree,
                        neighbor_code,
                        TWIN_VERTEX_INDEX[vertex_index.to_index()],
                        depth,
                    )),
                }
            }
        }
    }
    if nodes.is_empty().not() {
        let mut lods = HashSet::new();
        nodes.iter().all(|node| lods.insert(node.code.level()));
        assert!(lods.len() <= 2);
    }
    nodes
}

pub fn get_edge_neighbor_lod_octree_nodes<'a>(
    lod_octree: &'a TerrainLodOctree,
    current_node: &TerrainLodOctreeNode,
    edge_index: EdgeIndex,
    depth: u8,
) -> Vec<&'a TerrainLodOctreeNode> {
    let mut nodes = vec![];
    let neighbor_code = current_node.code.get_neighbor_edge_morton_code(edge_index);
    if let Ok(neighbor_code) = neighbor_code {
        let neighbor_node = lod_octree.get_node(&neighbor_code);
        match neighbor_node {
            Some(node) => {
                nodes.push(node);
            }
            None => {
                let parent_node = get_parent_lod_octree_node(lod_octree, &neighbor_code, depth);
                match parent_node {
                    Some(node) => {
                        nodes.push(node);
                    }
                    None => nodes.extend(get_children_lod_octree_nodes_by_edge(
                        lod_octree,
                        neighbor_code,
                        TWIN_EDGE_INDEX[edge_index.to_index()],
                        depth,
                    )),
                }
            }
        }
    }
    if nodes.is_empty().not() {
        let mut lods = HashSet::new();
        nodes.iter().all(|node| lods.insert(node.code.level()));
        assert!(lods.len() <= 2);
    }
    nodes
}

pub fn get_face_neighbor_lod_octree_nodes<'a>(
    lod_octree: &'a TerrainLodOctree,
    current_node: &TerrainLodOctreeNode,
    face_index: FaceIndex,
    depth: u8,
) -> Vec<&'a TerrainLodOctreeNode> {
    let mut nodes = vec![];

    let neighbor_code = current_node.code.get_neighbor_face_morton_code(face_index);
    if let Ok(neighbor_code) = neighbor_code {
        let neighbor_node = lod_octree.get_node(&neighbor_code);
        match neighbor_node {
            Some(node) => nodes.push(node),
            None => {
                let parent_node = get_parent_lod_octree_node(lod_octree, &neighbor_code, depth);
                match parent_node {
                    Some(node) => {
                        nodes.push(node);
                    }
                    None => nodes.extend(get_children_lod_octree_nodes_by_face(
                        lod_octree,
                        neighbor_code,
                        TWIN_FACE_INDEX[face_index.to_index()],
                        depth,
                    )),
                }
            }
        }
    }

    if nodes.is_empty().not() {
        let mut lods = HashSet::new();
        nodes.iter().all(|node| lods.insert(node.code.level()));
        assert!(lods.len() <= 2);
    }
    nodes
}

pub fn get_neighbor_positive_direction_nodes<'a>(
    lod_octree: &'a TerrainLodOctree,
    current_node: &'a TerrainLodOctreeNode,
    depth: u8,
) -> Vec<&'a TerrainLodOctreeNode> {
    let mut nodes = vec![];
    let right_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Right, depth);
    nodes.extend(right_face_nodes.iter());
    let front_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Front, depth);
    nodes.extend(front_face_nodes.iter());
    let top_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Top, depth);
    nodes.extend(top_face_nodes.iter());

    let x11_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::XAxisY1Z1, depth);
    nodes.extend(x11_axis_edge_nodes.iter());
    let y11_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::YAxisX1Z1, depth);
    nodes.extend(y11_axis_edge_nodes.iter());
    let z11_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::ZAxisX1Y1, depth);
    nodes.extend(z11_axis_edge_nodes.iter());

    // let vertex_111_nodes =
    //     get_vertex_neighbor_lod_octree_nodes(lod_octree, current_node, VertexIndex::X1Y1Z1, depth);
    // nodes.extend(vertex_111_nodes.iter());

    nodes
}

pub fn get_neighbor_negative_direction_nodes<'a>(
    lod_octree: &'a TerrainLodOctree,
    current_node: &'a TerrainLodOctreeNode,
    depth: u8,
) -> Vec<&'a TerrainLodOctreeNode> {
    let mut nodes = vec![];
    let left_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Left, depth);
    nodes.extend(left_face_nodes.iter());
    let back_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Back, depth);
    nodes.extend(back_face_nodes.iter());
    let bottom_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Bottom, depth);
    nodes.extend(bottom_face_nodes.iter());

    let x00_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::XAxisY0Z0, depth);
    nodes.extend(x00_axis_edge_nodes.iter());
    let y00_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::YAxisX0Z0, depth);
    nodes.extend(y00_axis_edge_nodes.iter());
    let z00_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::ZAxisX0Y0, depth);
    nodes.extend(z00_axis_edge_nodes.iter());

    let vertex_000_nodes =
        get_vertex_neighbor_lod_octree_nodes(lod_octree, current_node, VertexIndex::X0Y0Z0, depth);
    nodes.extend(vertex_000_nodes.iter());

    nodes
}

pub fn get_neighbor_all_nodes<'a>(
    lod_octree: &'a TerrainLodOctree,
    current_node: &'a TerrainLodOctreeNode,
    depth: u8,
) -> Vec<&'a TerrainLodOctreeNode> {
    let mut nodes = vec![];
    let right_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Right, depth);
    nodes.extend(right_face_nodes.iter());
    let front_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Front, depth);
    nodes.extend(front_face_nodes.iter());
    let top_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Top, depth);
    nodes.extend(top_face_nodes.iter());
    let left_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Left, depth);
    nodes.extend(left_face_nodes.iter());
    let back_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Back, depth);
    nodes.extend(back_face_nodes.iter());
    let bottom_face_nodes =
        get_face_neighbor_lod_octree_nodes(lod_octree, current_node, FaceIndex::Bottom, depth);
    nodes.extend(bottom_face_nodes.iter());

    let x11_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::XAxisY1Z1, depth);
    nodes.extend(x11_axis_edge_nodes.iter());
    let y11_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::YAxisX1Z1, depth);
    nodes.extend(y11_axis_edge_nodes.iter());
    let z11_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::ZAxisX1Y1, depth);
    nodes.extend(z11_axis_edge_nodes.iter());
    let x00_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::XAxisY0Z0, depth);
    nodes.extend(x00_axis_edge_nodes.iter());
    let y00_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::YAxisX0Z0, depth);
    nodes.extend(y00_axis_edge_nodes.iter());
    let z00_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::ZAxisX0Y0, depth);
    nodes.extend(z00_axis_edge_nodes.iter());
    let x01_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::XAxisY0Z1, depth);
    nodes.extend(x01_axis_edge_nodes.iter());
    let y01_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::YAxisX0Z1, depth);
    nodes.extend(y01_axis_edge_nodes.iter());
    let z01_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::ZAxisX0Y1, depth);
    nodes.extend(z01_axis_edge_nodes.iter());
    let x10_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::XAxisY1Z0, depth);
    nodes.extend(x10_axis_edge_nodes.iter());
    let y10_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::YAxisX1Z0, depth);
    nodes.extend(y10_axis_edge_nodes.iter());
    let z10_axis_edge_nodes =
        get_edge_neighbor_lod_octree_nodes(lod_octree, current_node, EdgeIndex::ZAxisX1Y0, depth);
    nodes.extend(z10_axis_edge_nodes.iter());

    // let vertex_000_nodes =
    //     get_vertex_neighbor_lod_octree_nodes(lod_octree, current_node, VertexIndex::X0Y0Z0, depth);
    // nodes.extend(vertex_000_nodes.iter());
    // let vertex_001_nodes =
    //     get_vertex_neighbor_lod_octree_nodes(lod_octree, current_node, VertexIndex::X0Y0Z1, depth);
    // nodes.extend(vertex_001_nodes.iter());
    // let vertex_010_nodes =
    //     get_vertex_neighbor_lod_octree_nodes(lod_octree, current_node, VertexIndex::X0Y1Z0, depth);
    // nodes.extend(vertex_010_nodes.iter());
    // let vertex_011_nodes =
    //     get_vertex_neighbor_lod_octree_nodes(lod_octree, current_node, VertexIndex::X0Y1Z1, depth);
    // nodes.extend(vertex_011_nodes.iter());
    // let vertex_100_nodes =
    //     get_vertex_neighbor_lod_octree_nodes(lod_octree, current_node, VertexIndex::X1Y0Z0, depth);
    // nodes.extend(vertex_100_nodes.iter());
    // let vertex_101_nodes =
    //     get_vertex_neighbor_lod_octree_nodes(lod_octree, current_node, VertexIndex::X1Y0Z1, depth);
    // nodes.extend(vertex_101_nodes.iter());
    // let vertex_110_nodes =
    //     get_vertex_neighbor_lod_octree_nodes(lod_octree, current_node, VertexIndex::X1Y1Z0, depth);
    // nodes.extend(vertex_110_nodes.iter());
    // let vertex_111_nodes =
    //     get_vertex_neighbor_lod_octree_nodes(lod_octree, current_node, VertexIndex::X1Y1Z1, depth);
    // nodes.extend(vertex_111_nodes.iter());

    nodes
}
