use std::fmt::Display;

use super::tables::{EdgeIndex, FaceIndex, SubCellIndex, VertexIndex, NEIGHBOUR_ADDRESS_TABLE};

use bevy::{log::info, reflect::Reflect, utils::hashbrown::HashMap};
use bitfield_struct::bitfield;
use ndshape::Shape;
use strum::EnumCount;

/// store octree cell address
// some CellAddress max bit is 48(usize) / 3 = 16. so max octree level is 16.
#[bitfield(u64)]
#[derive(PartialEq, Eq, Hash, Reflect)]
pub struct CellAddress {
    #[bits(48)]
    pub raw_address: u64,
    #[bits(16)]
    pub depth: u16,
}

impl Display for CellAddress {
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

impl CellAddress {
    pub fn set(&mut self, parent_address: CellAddress, pos_in_parent: SubCellIndex) {
        self.set_raw_address(parent_address.raw_address() << 3 | pos_in_parent as u64);
        self.set_depth(parent_address.depth() + 1);
    }

    pub fn set_child(&mut self, pos_in_parent: SubCellIndex) {
        self.set_raw_address(self.raw_address() << 3 | pos_in_parent as u64);
        self.set_depth(self.depth() + 1);
    }

    pub fn reset(&mut self) {
        self.set_raw_address(0);
        self.set_depth(0);
    }

    pub fn root() -> Self {
        CellAddress::new().with_raw_address(0).with_depth(1)
    }
}

impl CellAddress {
    pub fn get_parent_address(&self) -> CellAddress {
        let mut parent_address = *self;
        parent_address.set_raw_address(parent_address.raw_address() >> 3);
        parent_address.set_depth(parent_address.depth() - 1);
        parent_address
    }

    pub fn get_pos_in_parent(&self) -> Option<SubCellIndex> {
        if self.depth() <= 1 {
            return None;
        }

        let pos_in_parent = self.raw_address() & 0b111;
        SubCellIndex::from_repr(pos_in_parent as usize)
    }

    pub fn get_depth(&self) -> usize {
        self.depth() as usize
    }

    pub fn get_children_addresses(&self) -> [CellAddress; SubCellIndex::COUNT] {
        let mut children_address = [CellAddress::new(); SubCellIndex::COUNT];
        for (i, child) in children_address.iter_mut().enumerate() {
            child.set_raw_address(self.raw_address() << 3 | i as u64);
            child.set_depth(self.depth() + 1);
        }
        children_address
    }

    pub fn get_child_address(&self, sub_cell_index: SubCellIndex) -> CellAddress {
        let mut child_address = CellAddress::new();
        child_address.set_raw_address(self.raw_address() << 3 | sub_cell_index as u64);
        child_address.set_depth(self.depth() + 1);
        child_address
    }

    pub fn get_neighbour_address(&self, face_index: FaceIndex) -> CellAddress {
        let mut address = *self;

        let mut neighbour_address = CellAddress::new();
        let mut shift_count = 0;

        loop {
            let Some(sub_cell_index) = address.get_pos_in_parent() else {
                return address;
            };

            // if searching for right(+X), top(+Y) or front(+Z) neighbour
            // it should always have a greater slot value
            // if searching for left(-X), bottom(-Y) or back(-Z) neighbour
            // the neighbour should always have a smaller slot value,
            // OTHERWISE it means it belongs to a different parent
            let (neighbour_sub_cell_index, same_parent) =
                NEIGHBOUR_ADDRESS_TABLE[face_index as usize][sub_cell_index as usize];

            neighbour_address.set_raw_address(
                neighbour_address.raw_address() | (neighbour_sub_cell_index as u64) << shift_count,
            );
            neighbour_address.set_depth(neighbour_address.depth() + 1);

            address = address.get_parent_address();
            shift_count += 3;
            if same_parent {
                neighbour_address.set_raw_address(
                    neighbour_address.raw_address() | address.raw_address() << shift_count,
                );
                neighbour_address.set_depth(neighbour_address.depth() + 1);
                break;
            }

            if address.raw_address() == 0 {
                break;
            }
        }

        neighbour_address
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeAddress {
    pub cell_address: CellAddress,
    pub edge_index: EdgeIndex,
}

impl EdgeAddress {
    pub fn new(cell_address: CellAddress, edge_index: EdgeIndex) -> Self {
        Self {
            cell_address,
            edge_index,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FaceAddress {
    pub cell_address: CellAddress,
    pub face_index: FaceIndex,
}

impl FaceAddress {
    pub fn new(cell_address: CellAddress, face_index: FaceIndex) -> Self {
        Self {
            cell_address,
            face_index,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VertexAddress {
    pub cell_address: CellAddress,
    pub vertex_index: VertexIndex,
}

impl VertexAddress {
    pub fn new(cell_address: CellAddress, vertex_index: VertexIndex) -> Self {
        Self {
            cell_address,
            vertex_index,
        }
    }
}

pub fn construct_octree_address_map<S>(shape: &S) -> HashMap<u16, Vec<CellAddress>>
where
    S: Shape<3, Coord = u32>,
{
    let size = shape.as_array();
    assert_eq!(size[0], size[1]);
    assert_eq!(size[0], size[2]);
    let depth = (size[0] as f32).log2().ceil() as u16 + 1;

    let mut leaf_address_map: HashMap<u16, Vec<CellAddress>> = HashMap::default();
    let mut size = size[0];
    for i in 0..depth {
        if size == 0 {
            break;
        }
        let shape = ndshape::RuntimeShape::<u32, 3>::new([size, size, size]);
        let vec = construct_octree_address_vec(&shape);
        leaf_address_map.insert(depth - i, vec);
        size >>= 1;
    }
    leaf_address_map
}

pub fn construct_octree_address_vec<S>(shape: &S) -> Vec<CellAddress>
where
    S: Shape<3, Coord = u32>,
{
    let mut leaf_address_vec: Vec<CellAddress> = vec![CellAddress::root(); shape.usize()];

    let size = shape.as_array();
    assert_eq!(size[0], size[1]);
    assert_eq!(size[0], size[2]);

    let depth = (size[0] as f32).log2().ceil() as u32;

    for x in 0..size[0] {
        for y in 0..size[1] {
            for z in 0..size[2] {
                let mut half_size = [size[0] / 2, size[1] / 2, size[2] / 2];
                let index = shape.linearize([x, y, z]);
                let leaf_address = leaf_address_vec.get_mut(index as usize).unwrap();

                for i in 0..depth {
                    let half_offset = match (x < half_size[0], y < half_size[1], z < half_size[2]) {
                        (true, true, true) => {
                            leaf_address.set_child(SubCellIndex::X0Y0Z0);
                            [
                                -(size[0] as i32 / 2i32.pow(i + 2)),
                                -(size[1] as i32 / 2i32.pow(i + 2)),
                                -(size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                        (true, true, false) => {
                            leaf_address.set_child(SubCellIndex::X0Y0Z1);
                            [
                                -(size[0] as i32 / 2i32.pow(i + 2)),
                                -(size[1] as i32 / 2i32.pow(i + 2)),
                                (size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                        (true, false, true) => {
                            leaf_address.set_child(SubCellIndex::X0Y1Z0);
                            [
                                -(size[0] as i32 / 2i32.pow(i + 2)),
                                (size[1] as i32 / 2i32.pow(i + 2)),
                                -(size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                        (true, false, false) => {
                            leaf_address.set_child(SubCellIndex::X0Y1Z1);
                            [
                                -(size[0] as i32 / 2i32.pow(i + 2)),
                                (size[1] as i32 / 2i32.pow(i + 2)),
                                (size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                        (false, true, true) => {
                            leaf_address.set_child(SubCellIndex::X1Y0Z0);
                            [
                                (size[0] as i32 / 2i32.pow(i + 2)),
                                -(size[1] as i32 / 2i32.pow(i + 2)),
                                -(size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                        (false, true, false) => {
                            leaf_address.set_child(SubCellIndex::X1Y0Z1);
                            [
                                (size[0] as i32 / 2i32.pow(i + 2)),
                                -(size[1] as i32 / 2i32.pow(i + 2)),
                                (size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                        (false, false, true) => {
                            leaf_address.set_child(SubCellIndex::X1Y1Z0);
                            [
                                (size[0] as i32 / 2i32.pow(i + 2)),
                                (size[1] as i32 / 2i32.pow(i + 2)),
                                -(size[2] as i32 / 2i32.pow(i + 2)),
                            ]
                        }
                        (false, false, false) => {
                            leaf_address.set_child(SubCellIndex::X1Y1Z1);
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

    leaf_address_vec
}

#[cfg(test)]
mod tests {
    use bevy::utils::hashbrown::HashSet;

    use super::*;

    #[test]
    fn test_get_child_address() {
        let address = CellAddress::root();
        let child_address = address.get_child_address(SubCellIndex::X0Y0Z0);
        assert_eq!(
            child_address,
            CellAddress::new().with_raw_address(0).with_depth(2)
        );
    }

    #[test]
    fn test_get_pos_in_parent() {
        let address = CellAddress::root();
        let pos_in_parent = address.get_pos_in_parent();
        assert_eq!(pos_in_parent, None);

        let address = CellAddress::root();
        let child_address = address.get_child_address(SubCellIndex::X0Y1Z0);
        assert_eq!(
            child_address.get_pos_in_parent(),
            Some(SubCellIndex::X0Y1Z0)
        );
    }

    #[test]
    fn test_get_neighbour_address_simple() {
        let address = CellAddress::root();
        assert_eq!(
            address.get_neighbour_address(FaceIndex::Right),
            CellAddress::root()
        );

        let address = CellAddress::root();
        let child_address = address.get_child_address(SubCellIndex::X0Y0Z0);
        assert_eq!(
            child_address.get_neighbour_address(FaceIndex::Right),
            CellAddress::new().with_raw_address(1).with_depth(2)
        );
    }

    #[test]
    fn test_get_neighbour_address() {
        let address = CellAddress::root();
        let child_address_1 = address.get_child_address(SubCellIndex::X0Y0Z0);
        let child_address_2 = address.get_child_address(SubCellIndex::X0Y0Z1);
        assert_eq!(
            child_address_1.get_neighbour_address(FaceIndex::Front),
            child_address_2
        );
    }

    #[test]
    fn test_get_neighbour_address_crossing_cell() {
        let address = CellAddress::root();

        let child_address_1 = address.get_child_address(SubCellIndex::X1Y1Z0);
        let child_child_address_1 = child_address_1.get_child_address(SubCellIndex::X0Y0Z0);

        assert_eq!(
            child_child_address_1.get_neighbour_address(FaceIndex::Left),
            CellAddress::new().with_raw_address(0o021).with_depth(3)
        );

        let child_address_2 = address.get_child_address(SubCellIndex::X0Y1Z0);
        let child_child_address_2 = child_address_2.get_child_address(SubCellIndex::X1Y0Z0);

        assert_eq!(
            child_child_address_1.get_neighbour_address(FaceIndex::Left),
            child_child_address_2
        );
    }

    #[test]
    fn test_construct_octree_address_vec() {
        let shape = [2, 2, 2];
        let shape = ndshape::RuntimeShape::<u32, 3>::new(shape);
        let leaf_address = construct_octree_address_vec(&shape);
        assert_eq!(leaf_address.len(), 8);

        assert_eq!(
            leaf_address[0],
            CellAddress::new().with_depth(2).with_raw_address(0o0)
        );

        assert_eq!(
            leaf_address[4],
            CellAddress::new().with_depth(2).with_raw_address(0o04)
        );

        assert_eq!(
            leaf_address[5],
            CellAddress::new().with_depth(2).with_raw_address(0o05)
        );

        assert_eq!(
            leaf_address[2],
            CellAddress::new().with_depth(2).with_raw_address(0o02)
        );
    }

    #[test]
    fn test_construct_octree_address_map() {
        let shape = [16, 16, 16];
        let shape = ndshape::RuntimeShape::<u32, 3>::new(shape);
        let leaf_address_map = construct_octree_address_map(&shape);
        assert_eq!(leaf_address_map.len(), 5);

        assert!(leaf_address_map.get(&0).is_none());

        let leaf_address = leaf_address_map.get(&1).unwrap();
        assert_eq!(leaf_address.len(), 1);

        let leaf_address = leaf_address_map.get(&2).unwrap();
        assert_eq!(leaf_address.len(), 8_usize.pow(1));

        let leaf_address = leaf_address_map.get(&3).unwrap();
        assert_eq!(leaf_address.len(), 8_usize.pow(2));
        let mut check_set: HashSet<CellAddress> = HashSet::new();
        check_set.extend(leaf_address.iter());
        assert!(check_set.len() == leaf_address.len());

        let leaf_address = leaf_address_map.get(&4).unwrap();
        assert_eq!(leaf_address.len(), 8_usize.pow(3));
        let mut check_set: HashSet<CellAddress> = HashSet::new();
        check_set.extend(leaf_address.iter());
        assert!(check_set.len() == leaf_address.len());

        let leaf_address = leaf_address_map.get(&5).unwrap();
        assert_eq!(leaf_address.len(), 8_usize.pow(4));
        let mut check_set: HashSet<CellAddress> = HashSet::new();
        check_set.extend(leaf_address.iter());
        assert!(check_set.len() == leaf_address.len());

        assert!(leaf_address_map.get(&6).is_none());
    }
}
