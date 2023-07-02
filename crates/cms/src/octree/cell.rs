use bevy::prelude::*;
use std::{cell::RefCell, rc::Rc};

use super::{address::Address, tables::FaceIndex};
use nalgebra::Vector3;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CellType {
    Branch,
    Leaf, // 没有在表面的都不是leaf。
}

#[derive(Debug, Component, Default)]
pub struct CellMeshInfo {
    componnets: Vec<Vec<usize>>,
}

#[derive(Debug, Component)]
pub struct Cell {
    id: usize,

    cell_type: CellType,

    /// x, y and z
    size: Vector3<f32>,

    /// all corners sample index, global coord.
    corner_sample_index: [Vector3<usize>; 8],

    /// sample points cross size.
    sample_size: Vector3<usize>,

    /// first sample position
    c000: Vector3<usize>,

    address: Address,
}

impl Cell {
    pub fn new(
        id: usize,
        cell_type: CellType,
        address: Address,
        c000: Vector3<usize>,
        offset: Vector3<usize>,
        corner_sample_index: [Vector3<usize>; 8],
    ) -> Self {
        const INIT_CELL: Option<Rc<RefCell<Cell>>> = None;

        Self {
            id,
            cell_type,
            size: Vector3::new(0.0, 0.0, 0.0),
            corner_sample_index,
            sample_size: offset,
            c000,
            address,
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

    pub fn set_size(&mut self, size: Vector3<f32>) {
        self.size = size;
    }

    pub fn get_size(&self) -> &Vector3<f32> {
        &self.size
    }

    pub fn set_corner_sample_index(&mut self, corner_sample_index: [Vector3<usize>; 8]) {
        self.corner_sample_index = corner_sample_index;
    }

    pub fn get_corner_sample_index(&self) -> &[Vector3<usize>; 8] {
        &self.corner_sample_index
    }

    pub fn set_offsets_size(&mut self, offsets_size: Vector3<usize>) {
        self.sample_size = offsets_size;
    }

    pub fn get_offsets_size(&self) -> &Vector3<usize> {
        &self.sample_size
    }

    pub fn set_c000(&mut self, c000: Vector3<usize>) {
        self.c000 = c000;
    }

    pub fn get_c000(&self) -> &Vector3<usize> {
        &self.c000
    }

    pub fn set_address(&mut self, address: Address) {
        self.address = address;
    }

    pub fn get_address(&self) -> &Address {
        &self.address
    }

    pub fn get_id(&self) -> &usize {
        &self.id
    }
}

impl Cell {
    pub fn get_twin_face_address(&self, face_index: FaceIndex) -> (Address, FaceIndex) {
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
