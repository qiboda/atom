use super::tables::{EdgeIndex, FaceIndex, SubCellIndex, VertexIndex, NEIGHBOUR_ADDRESS_TABLE};

use bevy::reflect::Reflect;
use bitfield_struct::bitfield;
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

impl CellAddress {
    pub fn set(&mut self, parent_address: CellAddress, pos_in_parent: SubCellIndex) {
        self.set_raw_address(parent_address.raw_address() << 3 | pos_in_parent as u64);
        self.set_depth(parent_address.depth() + 1);
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

    pub fn get_children_address(&self) -> [CellAddress; SubCellIndex::COUNT] {
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

#[cfg(test)]
mod tests {
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
}
