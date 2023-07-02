use crate::octree::tables::SubCellIndex;

use super::tables::{FaceIndex, NEIGHBOUR_ADDRESS_TABLE};

use strum::EnumCount;

/// store octree cell address
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Address {
    pub raw_address: usize,
}

impl Address {
    pub fn new() -> Self {
        Self { raw_address: 0 }
    }

    pub fn set(&mut self, parent_address: Address, pos_in_parent: SubCellIndex) {
        self.raw_address = parent_address.raw_address << 3 | pos_in_parent as usize;
    }

    pub fn reset(&mut self) {
        self.raw_address = 0;
    }
}

impl Address {
    pub fn get_parent_address(&self) -> Address {
        let mut parent_address = self.clone();
        parent_address.raw_address >>= 3;
        parent_address
    }

    pub fn get_pos_in_parent(&self) -> SubCellIndex {
        let pos_in_parent = self.raw_address & 0b111;
        SubCellIndex::from(pos_in_parent)
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

    pub fn get_children_address(&self) -> [Address; SubCellIndex::COUNT] {
        let mut children_address = [Address::new(); SubCellIndex::COUNT];
        for (i, child) in children_address.iter_mut().enumerate() {
            child.raw_address = self.raw_address << 3 | i;
        }
        children_address
    }

    pub fn get_child_address(&self, sub_cell_index: SubCellIndex) -> Address {
        let mut child_address = Address::new();
        child_address.raw_address = self.raw_address << 3 | sub_cell_index as usize;
        child_address
    }

    /// todo: add test
    pub fn get_neighbour_address(&self, face_index: FaceIndex) -> Address {
        let mut address = self.clone();

        let mut neighbour_address = Address::new();
        let mut shift_count = 0;

        loop {
            let mut same_parent = false;
            let sub_cell_index = address.get_pos_in_parent();

            let neighbour_sub_cell_index =
                NEIGHBOUR_ADDRESS_TABLE[face_index as usize][sub_cell_index as usize];

            // if searching for right(+X), top(+Y) or front(+Z) neighbour
            // it should always have a greater slot value
            // if searching for left(-X), bottom(-Y) or back(-Z) neighbour
            // the neightbour should always have a smaller slot value,
            // OTHERWISE it means it belongs to a different parent
            match face_index {
                FaceIndex::Back | FaceIndex::Bottom | FaceIndex::Left => {
                    if neighbour_sub_cell_index < sub_cell_index {
                        same_parent = true;
                    }
                }
                FaceIndex::Front | FaceIndex::Top | FaceIndex::Right => {
                    if neighbour_sub_cell_index < sub_cell_index {
                        same_parent = true;
                    }
                }
            }

            if same_parent {
                neighbour_address.raw_address =
                    address.raw_address << shift_count | neighbour_address.raw_address;
                break;
            } else {
                neighbour_address.raw_address =
                    neighbour_address.raw_address | (neighbour_sub_cell_index << shift_count);
            }

            shift_count += 1;

            if address.raw_address == 0 {
                break;
            }

            address = address.get_parent_address();
        }

        neighbour_address
    }
}
