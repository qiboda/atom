use std::{cell::RefCell, rc::Rc};

use crate::address::Address;

use super::{
    face::Face,
    tables::{FaceIndex, SubCellIndex},
};
use nalgebra::Vector3;

use strum::EnumCount;

#[derive(Debug, PartialEq, Eq)]
pub enum CellType {
    Branch,
    Leaf,
}

pub struct Cell {
    id: usize,

    cell_type: CellType,

    parent: Option<Rc<RefCell<Cell>>>,

    neighbors: [Option<Rc<RefCell<Cell>>>; 6],

    children: Option<[Option<Rc<RefCell<Cell>>>; 8]>,

    componnets: Vec<Vec<usize>>,

    faces: [Rc<RefCell<Face>>; 6],

    cur_subdiv_level: u8,

    /// local coord
    pos_in_parent: Option<SubCellIndex>,

    /// x, y and z
    size: Vector3<f32>,

    /// geometry center
    /// todo: delete
    center: Vector3<f32>,

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
        parent_cell: Option<Rc<RefCell<Cell>>>,
        subdiv_level: u8,
        c000: Vector3<usize>,
        offset: Vector3<usize>,
        pos_in_parent: Option<SubCellIndex>,
    ) -> Self {
        const INIT_CELL: Option<Rc<RefCell<Cell>>> = None;

        let mut address = Address::new();
        match parent_cell.clone() {
            Some(parent) => address.set(parent.borrow().get_address().get_raw(), pos_in_parent),
            None => address.reset(),
        }

        let faces: [_; FaceIndex::COUNT] = [
            Rc::new(RefCell::new(Face::new(FaceIndex::Back, id))),
            Rc::new(RefCell::new(Face::new(FaceIndex::Front, id))),
            Rc::new(RefCell::new(Face::new(FaceIndex::Bottom, id))),
            Rc::new(RefCell::new(Face::new(FaceIndex::Top, id))),
            Rc::new(RefCell::new(Face::new(FaceIndex::Left, id))),
            Rc::new(RefCell::new(Face::new(FaceIndex::Right, id))),
        ];

        Self {
            id,
            cell_type,
            parent: parent_cell,
            neighbors: [INIT_CELL; 6],
            children: None,
            faces,
            cur_subdiv_level: subdiv_level,
            pos_in_parent,
            size: Vector3::new(0.0, 0.0, 0.0),
            center: Vector3::new(0.0, 0.0, 0.0),
            corner_sample_index: [Vector3::new(0, 0, 0); 8],
            sample_size: offset,
            c000,
            address,
            componnets: Vec::new(),
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

    pub fn set_parent(&mut self, parent: Option<Rc<RefCell<Cell>>>) {
        self.parent = parent;
    }

    pub fn get_parent(&self) -> &Option<Rc<RefCell<Cell>>> {
        &self.parent
    }

    pub fn set_neighbors(&mut self, neighbor: [Option<Rc<RefCell<Cell>>>; 6]) {
        self.neighbors = neighbor;
    }

    pub fn set_neighbor(&mut self, index: FaceIndex, neighbor: Option<Rc<RefCell<Cell>>>) {
        self.neighbors[index as usize] = neighbor;
    }

    pub fn get_neighbors(&self) -> &[Option<Rc<RefCell<Cell>>>; 6] {
        &self.neighbors
    }

    pub fn set_children(&mut self, children: Option<[Option<Rc<RefCell<Cell>>>; 8]>) {
        self.children = children;
    }

    pub fn get_children(&self) -> &Option<[Option<Rc<RefCell<Cell>>>; 8]> {
        &self.children
    }

    pub fn set_child(&mut self, index: usize, child: Option<Rc<RefCell<Cell>>>) {
        if let None = self.children.as_mut() {
            const INIT_CELL: Option<Rc<RefCell<Cell>>> = None;
            self.children = Some([INIT_CELL; 8]);
        }

        self.children.as_mut().unwrap()[index] = child;
    }

    pub fn get_child(&self, index: SubCellIndex) -> Option<Rc<RefCell<Cell>>> {
        if let Some(children) = self.children.as_ref() {
            return children[index as usize].clone();
        }

        None
    }

    pub fn set_faces(&mut self, faces: [Rc<RefCell<Face>>; 6]) {
        self.faces = faces;
    }

    pub fn get_faces(&self) -> &[Rc<RefCell<Face>>; 6] {
        &self.faces
    }

    pub fn get_face(&self, index: FaceIndex) -> Rc<RefCell<Face>> {
        self.get_faces()[index as usize].clone()
    }

    pub fn set_cur_subdiv_level(&mut self, cur_subdiv_level: u8) {
        self.cur_subdiv_level = cur_subdiv_level;
    }

    pub fn get_cur_subdiv_level(&self) -> &u8 {
        &self.cur_subdiv_level
    }

    pub fn set_pos_in_parent(&mut self, pos_in_parent: Option<SubCellIndex>) {
        self.pos_in_parent = pos_in_parent;
    }

    pub fn get_pos_in_parent(&self) -> &Option<SubCellIndex> {
        &self.pos_in_parent
    }

    pub fn set_size(&mut self, size: Vector3<f32>) {
        self.size = size;
    }

    pub fn get_size(&self) -> &Vector3<f32> {
        &self.size
    }

    pub fn set_center(&mut self, center: Vector3<f32>) {
        self.center = center;
    }

    pub fn get_center(&self) -> &Vector3<f32> {
        &self.center
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

    pub fn set_componnets(&mut self, componnets: Vec<Vec<usize>>) {
        self.componnets = componnets;
    }

    pub fn get_componnets(&self) -> &Vec<Vec<usize>> {
        &self.componnets
    }

    pub fn get_componnets_mut(&mut self) -> &mut Vec<Vec<usize>> {
        &mut self.componnets
    }

    pub fn get_id(&self) -> &usize {
        &self.id
    }
}
