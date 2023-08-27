use bevy::prelude::*;

use super::{
    address::VoxelAddress,
    face::{FaceType, Faces},
    tables::FaceIndex,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CellType {
    Branch,
    Leaf, // 没有在表面的都不是leaf。
}

#[derive(Debug, Component)]
pub struct Cell {
    cell_type: CellType,

    pub faces: Faces,

    /// all corners sample index, global coord.
    corner_sample_index: [UVec3; 8],

    address: VoxelAddress,

    pub components: Option<Vec<Vec<u32>>>,
}

impl Cell {
    pub fn new(
        cell_type: CellType,
        face_type: FaceType,
        address: VoxelAddress,
        corner_sample_index: [UVec3; 8],
    ) -> Self {
        Self {
            cell_type,
            faces: Faces::new(face_type),
            corner_sample_index,
            address,
            components: None,
        }
    }
}

impl Cell {
    pub fn set_cell_type(&mut self, cell_type: CellType) {
        self.cell_type = cell_type;
    }

    pub fn get_cell_type(&self) -> &CellType {
        &self.cell_type
    }

    pub fn set_corner_sample_index(&mut self, corner_sample_index: [UVec3; 8]) {
        self.corner_sample_index = corner_sample_index;
    }

    pub fn get_corner_sample_index(&self) -> &[UVec3; 8] {
        &self.corner_sample_index
    }

    pub fn set_address(&mut self, address: VoxelAddress) {
        self.address = address;
    }

    pub fn get_address(&self) -> &VoxelAddress {
        &self.address
    }
}

impl Cell {
    pub fn get_twin_face_address(&self, face_index: FaceIndex) -> (VoxelAddress, FaceIndex) {
        let neighbour_address = self.address.get_neighbour_address(face_index);
        let neighbour_face_index = match face_index {
            FaceIndex::Back => FaceIndex::Front,
            FaceIndex::Front => FaceIndex::Back,
            FaceIndex::Bottom => FaceIndex::Top,
            FaceIndex::Top => FaceIndex::Bottom,
            FaceIndex::Left => FaceIndex::Right,
            FaceIndex::Right => FaceIndex::Left,
        };
        (neighbour_address, neighbour_face_index)
    }
}
