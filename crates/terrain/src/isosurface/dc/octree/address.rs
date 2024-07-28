use std::fmt::{Debug, Display};

use crate::chunk_mgr::chunk::chunk_lod::OctreeDepthType;

use super::tables::{
    EdgeIndex, FaceIndex, SubNodeIndex, VertexIndex, EDGE_SUBNODE_COUNT, EDGE_SUBNODE_PAIRS,
    FACE_SUBNODE_COUNT, NEIGHBOR_ADDRESS_TABLE, NEIGHBOR_NODE_IN_EDGE, NEIGHBOR_NODE_IN_VERTEX,
    SUBNODE_IN_FACE,
};

use bevy::{reflect::Reflect, utils::hashbrown::HashMap};
use bitfield_struct::bitfield;
use ndshape::{RuntimeShape, Shape};
use strum::EnumCount;
use tracing::info;

/// store octree node address
// some NodeAddress max bit is 48(usize) / 3 = 16. so max octree level is 16.
#[bitfield(u64, debug = false)]
#[derive(PartialEq, Eq, Hash, Reflect)]
pub struct NodeAddress {
    #[bits(48)]
    pub raw_address: u64,
    #[bits(16)]
    pub depth: u16,
}

impl Display for NodeAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "address: {:#0width$o}#{}",
            self.raw_address(),
            self.depth(),
            width = self.depth() as usize,
        )
    }
}

impl Debug for NodeAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "address: {:#0width$o}#{}",
            self.raw_address(),
            self.depth(),
            width = self.depth() as usize,
        )
    }
}

impl NodeAddress {
    pub fn set(&mut self, parent_address: NodeAddress, pos_in_parent: SubNodeIndex) {
        self.set_raw_address(parent_address.raw_address() << 3 | pos_in_parent as u64);
        self.set_depth(parent_address.depth() + 1);
    }

    pub fn set_child(&mut self, pos_in_parent: SubNodeIndex) {
        self.set_raw_address(self.raw_address() << 3 | pos_in_parent as u64);
        self.set_depth(self.depth() + 1);
    }

    pub fn concat_address(&self, node_address: NodeAddress) -> NodeAddress {
        let mut address = *self;
        let depth_offset = node_address.depth();
        address.set_raw_address(
            address.raw_address() << (depth_offset * 3) | node_address.raw_address(),
        );
        address.set_depth(address.depth() + depth_offset);
        address
    }

    pub fn reset(&mut self) {
        self.set_raw_address(0);
        self.set_depth(0);
    }

    pub fn root() -> Self {
        NodeAddress::new().with_raw_address(0).with_depth(0)
    }
}

impl NodeAddress {
    pub fn get_parent_address(&self) -> NodeAddress {
        let mut parent_address = *self;
        parent_address.set_raw_address(parent_address.raw_address() >> 3);
        parent_address.set_depth(parent_address.depth() - 1);
        parent_address
    }

    pub fn get_pos_by_depth(&self, depth: u16) -> Option<SubNodeIndex> {
        if self.depth() < depth {
            return None;
        }

        let mut pos_in_parent = self.raw_address() & (0b111 << ((self.depth() - depth) * 3));
        pos_in_parent >>= (self.depth() - depth) * 3;
        SubNodeIndex::from_repr(pos_in_parent as usize)
    }

    pub fn get_pos_in_parent(&self) -> Option<SubNodeIndex> {
        if self.depth() == 0 {
            return None;
        }
        self.get_pos_by_depth(self.depth())
    }

    pub fn get_depth(&self) -> OctreeDepthType {
        self.depth() as OctreeDepthType
    }

    pub fn get_children_addresses(&self) -> [NodeAddress; SubNodeIndex::COUNT] {
        let mut children_address = [NodeAddress::new(); SubNodeIndex::COUNT];
        for (i, child) in children_address.iter_mut().enumerate() {
            child.set_raw_address(self.raw_address() << 3 | i as u64);
            child.set_depth(self.depth() + 1);
        }
        children_address
    }

    pub fn get_children_addresses_by_face(
        &self,
        face_index: FaceIndex,
    ) -> [NodeAddress; FACE_SUBNODE_COUNT] {
        let mut children_address = [NodeAddress::new(); FACE_SUBNODE_COUNT];
        for (index, sub_node_index) in SUBNODE_IN_FACE[face_index as usize].iter().enumerate() {
            children_address[index] = self.get_child_address(*sub_node_index);
        }
        children_address
    }

    pub fn get_children_addresses_by_edge(
        &self,
        edge_index: EdgeIndex,
    ) -> [NodeAddress; EDGE_SUBNODE_COUNT] {
        let mut children_address = [NodeAddress::new(); EDGE_SUBNODE_COUNT];
        for (index, sub_node_index) in EDGE_SUBNODE_PAIRS[edge_index as usize].iter().enumerate() {
            children_address[index] = self.get_child_address(*sub_node_index);
        }
        children_address
    }

    pub fn get_children_addresses_by_vertex(&self, vertex_index: VertexIndex) -> NodeAddress {
        self.get_child_address(vertex_index)
    }

    pub fn get_child_address(&self, sub_node_index: SubNodeIndex) -> NodeAddress {
        let mut child_address = NodeAddress::new();
        child_address.set_raw_address(self.raw_address() << 3 | sub_node_index as u64);
        child_address.set_depth(self.depth() + 1);
        child_address
    }

    pub fn get_edge_neighbor_address(&self, edge_index: EdgeIndex) -> NodeAddress {
        let mut neighbor_address = *self;
        for face_index in NEIGHBOR_NODE_IN_EDGE[edge_index as usize] {
            neighbor_address = neighbor_address.get_face_neighbor_address(face_index);
        }
        neighbor_address
    }

    pub fn get_vertex_neighbor_address(&self, vertex_index: VertexIndex) -> NodeAddress {
        let mut neighbor_address = *self;
        for face_index in NEIGHBOR_NODE_IN_VERTEX[vertex_index as usize] {
            neighbor_address = self.get_face_neighbor_address(face_index);
        }
        neighbor_address
    }

    pub fn get_face_neighbor_address(&self, face_index: FaceIndex) -> NodeAddress {
        let mut address = *self;

        let mut neighbor_address = NodeAddress::new();
        let mut shift_count = 0;

        loop {
            let Some(sub_node_index) = address.get_pos_in_parent() else {
                return address;
            };

            // if searching for right(+X), top(+Y) or front(+Z) neighbor
            // it should always have a greater slot value
            // if searching for left(-X), bottom(-Y) or back(-Z) neighbor
            // the neighbor should always have a smaller slot value,
            // OTHERWISE it means it belongs to a different parent
            let (neighbor_sub_node_index, same_parent) =
                NEIGHBOR_ADDRESS_TABLE[face_index as usize][sub_node_index as usize];

            neighbor_address.set_raw_address(
                neighbor_address.raw_address() | (neighbor_sub_node_index as u64) << shift_count,
            );
            neighbor_address.set_depth(neighbor_address.depth() + 1);

            address = address.get_parent_address();
            shift_count += 3;
            if same_parent {
                neighbor_address.set_raw_address(
                    neighbor_address.raw_address() | address.raw_address() << shift_count,
                );
                neighbor_address.set_depth(neighbor_address.depth() + address.depth());
                break;
            }
        }

        neighbor_address
    }
}

pub type CoordAddressVec = Vec<NodeAddress>;
pub type DepthCoordMap = HashMap<OctreeDepthType, CoordAddressVec>;

pub fn construct_octree_depth_coord_map(chunk_size: f32, voxel_size: f32) -> DepthCoordMap {
    // 此处乘以8是因为考虑到缝合边缘，衔接边缘的八个Chunk最大会相差三个lod。
    let size = (chunk_size * 8.0 / voxel_size) as u32;
    let shape = RuntimeShape::<u32, 3>::new([size, size, size]);

    let size = shape.as_array();
    assert_eq!(size[0], size[1]);
    assert_eq!(size[0], size[2]);
    let depth = (size[0] as f32).log2() as OctreeDepthType;

    info!("depth: {}, voxel: {}, size: {}", depth, voxel_size, size[0]);

    let mut depth_coord_map: DepthCoordMap = HashMap::default();
    let mut size = size[0];
    for i in (0..=depth).rev() {
        if size == 0 {
            break;
        }
        let shape = ndshape::RuntimeShape::<u32, 3>::new([size, size, size]);
        let vec = construct_octree_coord_address_vec(&shape);
        depth_coord_map.insert(i, vec);
        size >>= 1;
    }
    depth_coord_map
}

pub fn construct_octree_coord_address_vec<S>(shape: &S) -> CoordAddressVec
where
    S: Shape<3, Coord = u32>,
{
    let mut coord_address_vec: CoordAddressVec = vec![NodeAddress::root(); shape.usize()];

    let size = shape.as_array();
    assert_eq!(size[0], size[1]);
    assert_eq!(size[0], size[2]);

    let depth = (size[0] as f32).log2().ceil() as u32;

    for x in 0..size[0] {
        for y in 0..size[1] {
            for z in 0..size[2] {
                let mut half_size = [size[0] / 2, size[1] / 2, size[2] / 2];
                let index = shape.linearize([x, y, z]);
                let leaf_address = coord_address_vec.get_mut(index as usize).unwrap();

                for i in 0..depth {
                    let half_offset = match (x < half_size[0], y < half_size[1], z < half_size[2]) {
                        (true, true, true) => {
                            leaf_address.set_child(SubNodeIndex::X0Y0Z0);
                            [
                                -(size[0] as i32 / 2i32.pow(i + 2)),
                                -(size[1] as i32 / 2i32.pow(i + 2)),
                                -(size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                        (true, true, false) => {
                            leaf_address.set_child(SubNodeIndex::X0Y0Z1);
                            [
                                -(size[0] as i32 / 2i32.pow(i + 2)),
                                -(size[1] as i32 / 2i32.pow(i + 2)),
                                (size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                        (true, false, true) => {
                            leaf_address.set_child(SubNodeIndex::X0Y1Z0);
                            [
                                -(size[0] as i32 / 2i32.pow(i + 2)),
                                (size[1] as i32 / 2i32.pow(i + 2)),
                                -(size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                        (true, false, false) => {
                            leaf_address.set_child(SubNodeIndex::X0Y1Z1);
                            [
                                -(size[0] as i32 / 2i32.pow(i + 2)),
                                (size[1] as i32 / 2i32.pow(i + 2)),
                                (size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                        (false, true, true) => {
                            leaf_address.set_child(SubNodeIndex::X1Y0Z0);
                            [
                                (size[0] as i32 / 2i32.pow(i + 2)),
                                -(size[1] as i32 / 2i32.pow(i + 2)),
                                -(size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                        (false, true, false) => {
                            leaf_address.set_child(SubNodeIndex::X1Y0Z1);
                            [
                                (size[0] as i32 / 2i32.pow(i + 2)),
                                -(size[1] as i32 / 2i32.pow(i + 2)),
                                (size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                        (false, false, true) => {
                            leaf_address.set_child(SubNodeIndex::X1Y1Z0);
                            [
                                (size[0] as i32 / 2i32.pow(i + 2)),
                                (size[1] as i32 / 2i32.pow(i + 2)),
                                -(size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                        (false, false, false) => {
                            leaf_address.set_child(SubNodeIndex::X1Y1Z1);
                            [
                                (size[0] as i32 / 2i32.pow(i + 2)),
                                (size[1] as i32 / 2i32.pow(i + 2)),
                                (size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                    };

                    half_size = [
                        (half_size[0] as i32 + half_offset[0]) as u32,
                        (half_size[1] as i32 + half_offset[1]) as u32,
                        (half_size[2] as i32 + half_offset[2]) as u32,
                    ];
                }
            }
        }
    }

    coord_address_vec
}

#[cfg(test)]
mod tests {
    use bevy::utils::hashbrown::HashSet;

    use super::*;

    #[test]
    fn test_get_child_address() {
        let address = NodeAddress::root();
        let child_address = address.get_child_address(SubNodeIndex::X0Y0Z0);
        assert_eq!(
            child_address,
            NodeAddress::new().with_raw_address(0).with_depth(1)
        );
    }

    #[test]
    fn test_get_pos_in_parent() {
        let address = NodeAddress::root();
        let pos_in_parent = address.get_pos_in_parent();
        assert_eq!(pos_in_parent, None);

        let address = NodeAddress::root();
        let child_address = address.get_child_address(SubNodeIndex::X0Y1Z0);
        assert_eq!(
            child_address.get_pos_in_parent(),
            Some(SubNodeIndex::X0Y1Z0)
        );
    }

    #[test]
    fn test_get_pos_by_depth() {
        let address = NodeAddress::root();
        let pos = address.get_pos_by_depth(0);
        assert_eq!(pos, Some(SubNodeIndex::X0Y0Z0));

        let child_address = address.get_child_address(SubNodeIndex::X0Y1Z0);
        assert_eq!(
            child_address.get_pos_by_depth(0),
            Some(SubNodeIndex::X0Y0Z0)
        );
        assert_eq!(
            child_address.get_pos_by_depth(1),
            Some(SubNodeIndex::X0Y1Z0)
        );

        let mut address = NodeAddress::new();
        address.set_depth(4);
        address.set_raw_address(0o34);

        assert_eq!(address.get_pos_by_depth(1), Some(SubNodeIndex::X0Y0Z0));
        assert_eq!(address.get_pos_by_depth(3), SubNodeIndex::from_repr(3));
    }

    #[test]
    fn test_get_neighbor_address_simple() {
        let address = NodeAddress::root();
        assert_eq!(
            address.get_face_neighbor_address(FaceIndex::Right),
            NodeAddress::root()
        );

        let address = NodeAddress::root();
        let child_address = address.get_child_address(SubNodeIndex::X0Y0Z0);
        assert_eq!(
            child_address.get_face_neighbor_address(FaceIndex::Right),
            NodeAddress::new().with_raw_address(1).with_depth(1)
        );
    }

    #[test]
    fn test_get_neighbor_address() {
        let address = NodeAddress::root();
        let child_address_1 = address.get_child_address(SubNodeIndex::X0Y0Z0);
        let child_address_2 = address.get_child_address(SubNodeIndex::X0Y0Z1);
        assert_eq!(
            child_address_1.get_face_neighbor_address(FaceIndex::Front),
            child_address_2
        );
    }

    #[test]
    fn test_get_neighbor_address_crossing_node() {
        let address = NodeAddress::root();

        let child_address_1 = address.get_child_address(SubNodeIndex::X1Y1Z0);
        let child_child_address_1 = child_address_1.get_child_address(SubNodeIndex::X0Y0Z0);

        assert_eq!(
            child_child_address_1.get_face_neighbor_address(FaceIndex::Left),
            NodeAddress::new().with_raw_address(0o021).with_depth(2)
        );

        let child_address_2 = address.get_child_address(SubNodeIndex::X0Y1Z0);
        let child_child_address_2 = child_address_2.get_child_address(SubNodeIndex::X1Y0Z0);

        assert_eq!(
            child_child_address_1.get_face_neighbor_address(FaceIndex::Left),
            child_child_address_2
        );
    }

    #[test]
    fn test_neighbor_crossing_node() {
        let address = NodeAddress::new().with_raw_address(0o7).with_depth(2);
        let address_neighbor = address.get_face_neighbor_address(FaceIndex::Right);
        assert_eq!(
            address_neighbor,
            NodeAddress::new().with_raw_address(0o16).with_depth(2)
        );
    }

    #[test]
    fn test_construct_octree_address_vec() {
        // depth == 1
        let shape = [2, 2, 2];
        let shape = ndshape::RuntimeShape::<u32, 3>::new(shape);
        let leaf_address = construct_octree_coord_address_vec(&shape);
        assert_eq!(leaf_address.len(), 8);

        assert_eq!(
            leaf_address[0],
            NodeAddress::new().with_depth(1).with_raw_address(0o0)
        );

        assert_eq!(
            leaf_address[4],
            NodeAddress::new().with_depth(1).with_raw_address(0o04)
        );

        assert_eq!(
            leaf_address[5],
            NodeAddress::new().with_depth(1).with_raw_address(0o05)
        );

        assert_eq!(
            leaf_address[2],
            NodeAddress::new().with_depth(1).with_raw_address(0o02)
        );
    }

    #[test]
    fn test_construct_octree_address_map() {
        let leaf_address_map = construct_octree_depth_coord_map(4.0, 0.5);
        // 4, 2, 1, 0.5
        assert_eq!(leaf_address_map.len(), 7);

        let leaf_address = leaf_address_map.get(&0).unwrap();
        assert_eq!(leaf_address.len(), 1);
        assert_eq!(leaf_address[0].depth(), 0);

        let leaf_address = leaf_address_map.get(&1).unwrap();
        assert_eq!(leaf_address.len(), 8_usize.pow(1));
        assert_eq!(leaf_address[0].depth(), 1);

        let leaf_address = leaf_address_map.get(&2).unwrap();
        assert_eq!(leaf_address.len(), 8_usize.pow(2));
        assert_eq!(leaf_address[0].depth(), 2);
        let mut check_set: HashSet<NodeAddress> = HashSet::new();
        check_set.extend(leaf_address.iter());
        assert!(check_set.len() == leaf_address.len());

        let leaf_address = leaf_address_map.get(&3).unwrap();
        assert_eq!(leaf_address.len(), 8_usize.pow(3));
        assert_eq!(leaf_address[0].depth(), 3);
        let mut check_set: HashSet<NodeAddress> = HashSet::new();
        check_set.extend(leaf_address.iter());
        assert!(check_set.len() == leaf_address.len());

        let leaf_address = leaf_address_map.get(&4).unwrap();
        assert_eq!(leaf_address.len(), 8_usize.pow(4));
        assert_eq!(leaf_address[0].depth(), 4);
        let mut check_set: HashSet<NodeAddress> = HashSet::new();
        check_set.extend(leaf_address.iter());
        assert!(check_set.len() == leaf_address.len());

        assert!(leaf_address_map.get(&6).is_some());
        assert!(leaf_address_map.get(&7).is_none());
    }

    #[test]
    fn test_concat_address() {
        let address = NodeAddress::root();
        let child_address_1 = address.get_child_address(SubNodeIndex::X0Y1Z1);

        let address = NodeAddress::root();
        let child_address_2 = address.get_child_address(SubNodeIndex::X0Y1Z0);

        let concatenate_address = child_address_1.concat_address(child_address_2);

        let address = NodeAddress::root();
        let child_address = address.get_child_address(SubNodeIndex::X0Y1Z1);
        let child_child_address = child_address.get_child_address(SubNodeIndex::X0Y1Z0);
        assert_eq!(concatenate_address, child_child_address);

        let address_1 = NodeAddress::root();
        let address_2 = NodeAddress::root();
        assert_eq!(address_1.concat_address(address_2), NodeAddress::root());
    }
}
