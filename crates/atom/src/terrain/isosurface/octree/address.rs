use super::tables::{FaceIndex, SubCellIndex, NEIGHBOUR_ADDRESS_TABLE};

use bevy::prelude::info;
use strum::EnumCount;

/// store octree cell address
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VoxelAddress {
    pub raw_address: usize,
}

impl VoxelAddress {
    pub fn new() -> Self {
        Self { raw_address: 0 }
    }

    pub fn set(&mut self, parent_address: VoxelAddress, pos_in_parent: SubCellIndex) {
        self.raw_address = parent_address.raw_address << 3 | pos_in_parent as usize;
    }

    pub fn reset(&mut self) {
        self.raw_address = 0;
    }
}

impl VoxelAddress {
    pub fn get_parent_address(&self) -> VoxelAddress {
        let mut parent_address = self.clone();
        parent_address.raw_address >>= 3;
        parent_address
    }

    pub fn get_pos_in_parent(&self) -> SubCellIndex {
        let pos_in_parent = self.raw_address & 0b111;
        SubCellIndex::from_repr(pos_in_parent).unwrap()
    }

    pub fn get_level(&self) -> usize {
        let mut level = 0;
        let mut address = self.clone();
        while address.raw_address != 0 {
            address.raw_address >>= 3;
            level += 1;
        }
        level
    }

    pub fn get_children_address(&self) -> [VoxelAddress; SubCellIndex::COUNT] {
        let mut children_address = [VoxelAddress::new(); SubCellIndex::COUNT];
        for (i, child) in children_address.iter_mut().enumerate() {
            child.raw_address = self.raw_address << 3 | i;
        }
        children_address
    }

    pub fn get_child_address(&self, sub_cell_index: SubCellIndex) -> VoxelAddress {
        let mut child_address = VoxelAddress::new();
        child_address.raw_address = self.raw_address << 3 | sub_cell_index as usize;
        child_address
    }

    /// todo: add test
    pub fn get_neighbour_address(&self, face_index: FaceIndex) -> VoxelAddress {
        let mut address = self.clone();

        let mut neighbour_address = VoxelAddress::new();
        let mut shift_count = 0;

        loop {
            let sub_cell_index = address.get_pos_in_parent();

            // if searching for right(+X), top(+Y) or front(+Z) neighbour
            // it should always have a greater slot value
            // if searching for left(-X), bottom(-Y) or back(-Z) neighbour
            // the neightbour should always have a smaller slot value,
            // OTHERWISE it means it belongs to a different parent
            let (neighbour_sub_cell_index, same_parent) =
                NEIGHBOUR_ADDRESS_TABLE[face_index as usize][sub_cell_index as usize];

            info!("nighbour sub cell index: {:?}", neighbour_sub_cell_index);
            println!(
                "sub_cell_index {:?}, face_index: {:?}, nighbour sub cell index: {:?}, same parent : {}",
                sub_cell_index, face_index, neighbour_sub_cell_index, same_parent
            );

            println!("neighbour_address: {:o}", neighbour_address.raw_address);

            neighbour_address.raw_address = neighbour_address.raw_address
                | ((neighbour_sub_cell_index as usize) << shift_count);

            println!("neighbour_address: {:o}", neighbour_address.raw_address);
            println!("address: {:o}", address.raw_address);

            address = address.get_parent_address();
            shift_count += 3;
            println!("address: {:o}", address.raw_address);
            if same_parent {
                neighbour_address.raw_address =
                    (address.raw_address << shift_count) | neighbour_address.raw_address;
                break;
            }

            if address.raw_address == 0 {
                break;
            }
        }

        neighbour_address
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_child_address() {
        let address = VoxelAddress { raw_address: 1 };
        let child_address = address.get_child_address(SubCellIndex::LeftBottomBack);
        assert_eq!(
            child_address,
            VoxelAddress {
                raw_address: 0b1000
            }
        );
    }

    #[test]
    fn test_get_pos_in_parent() {
        let address = VoxelAddress { raw_address: 1 };
        let pos_in_parent = address.get_pos_in_parent();
        assert_eq!(pos_in_parent, SubCellIndex::LeftBottomFront);
    }

    #[test]
    fn test_get_neighbour_address_simple() {
        let address = VoxelAddress { raw_address: 1 };
        assert_eq!(
            address.get_neighbour_address(FaceIndex::Right),
            VoxelAddress { raw_address: 0b101 }
        );
    }

    #[test]
    fn test_get_neighbour_address() {
        let address = VoxelAddress { raw_address: 1 };
        let child_address_1 = address.get_child_address(SubCellIndex::LeftBottomBack);
        let child_address_2 = address.get_child_address(SubCellIndex::LeftBottomFront);
        assert_eq!(
            child_address_1.get_neighbour_address(FaceIndex::Front),
            child_address_2
        );
    }

    #[test]
    fn test_get_top_neighbour_address() {
        let address = VoxelAddress { raw_address: 1 };
        assert_eq!(
            address.get_neighbour_address(FaceIndex::Front),
            VoxelAddress { raw_address: 0o0 }
        );
    }

    #[test]
    fn test_get_neighbour_address_crossing_cell() {
        let address = VoxelAddress { raw_address: 1 };

        let child_address_1 = address.get_child_address(SubCellIndex::RightTopBack);
        let child_child_address_1 = child_address_1.get_child_address(SubCellIndex::LeftBottomBack);

        assert_eq!(
            child_child_address_1.get_neighbour_address(FaceIndex::Left),
            VoxelAddress { raw_address: 0o124 }
        );

        let child_address_2 = address.get_child_address(SubCellIndex::LeftTopBack);
        let child_child_address_2 =
            child_address_2.get_child_address(SubCellIndex::RightBottomBack);

        assert_eq!(
            child_child_address_1.get_neighbour_address(FaceIndex::Left),
            child_child_address_2
        );
    }
}
