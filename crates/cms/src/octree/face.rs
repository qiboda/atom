use std::{cell::RefCell, rc::Rc};

use bevy::prelude::*;

use super::{
    strip::Strip,
    tables::{FaceIndex, SubFaceIndex},
};

#[derive(Debug, PartialEq, Eq)]
pub enum FaceType {
    BranchFace,
    LeafFace,    // only cell's face can be leaf face.
    TransitFace, // 如果两个相邻的cell的face不是同一级的，那么这个face就是transit face
}

#[derive(Debug)]
pub struct Face {
    cell_id: usize,

    face_index: FaceIndex,

    face_type: FaceType,

    strips: Vec<Strip>,

    transit_segs: Vec<Vec<usize>>,
}

impl Face {
    pub fn new(face_index: FaceIndex, cell_id: usize, face_type: FaceType) -> Self {
        Self {
            cell_id,
            face_index,
            face_type,
            strips: Vec::new(),
            transit_segs: Vec::new(),
        }
    }
}

impl Face {
    pub fn get_cell_id(&self) -> usize {
        self.cell_id
    }

    pub fn get_face_index(&self) -> FaceIndex {
        self.face_index
    }

    pub fn set_face_type(&mut self, face_type: FaceType) {
        self.face_type = face_type;
    }

    pub fn get_face_type(&self) -> &FaceType {
        &self.face_type
    }

    pub fn set_strips(&mut self, strips: Vec<Strip>) {
        self.strips = strips;
    }

    pub fn get_strips(&self) -> &Vec<Strip> {
        &self.strips
    }

    pub fn get_strips_mut(&mut self) -> &mut Vec<Strip> {
        &mut self.strips
    }

    pub fn set_transit_segs(&mut self, transit_segs: Vec<Vec<usize>>) {
        self.transit_segs = transit_segs;
    }

    pub fn get_transit_segs(&self) -> &Vec<Vec<usize>> {
        &self.transit_segs
    }
}

#[derive(Debug, Component)]
pub struct Faces {
    faces: [Face; 6],
}

impl Faces {
    pub fn new(cell_id: usize, face_type: FaceType) -> Self {
        Self {
            faces: [
                Face::new(FaceIndex::Back, cell_id, face_type),
                Face::new(FaceIndex::Front, cell_id, face_type),
                Face::new(FaceIndex::Bottom, cell_id, face_type),
                Face::new(FaceIndex::Top, cell_id, face_type),
                Face::new(FaceIndex::Left, cell_id, face_type),
                Face::new(FaceIndex::Right, cell_id, face_type),
            ],
        }
    }
}

impl Faces {
    pub fn get_face(&self, face_index: FaceIndex) -> &Face {
        &self.faces[face_index as usize]
    }

    pub fn get_face_mut(&mut self, face_index: FaceIndex) -> &mut Face {
        &mut self.faces[face_index as usize]
    }
}
