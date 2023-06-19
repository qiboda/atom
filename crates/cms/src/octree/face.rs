use std::{cell::RefCell, rc::Rc};

use nalgebra::Vector3;

use super::{
    strip::Strip,
    tables::{FaceIndex, SubFaceIndex},
};

#[derive(Debug, PartialEq, Eq)]
pub enum FaceType {
    BranchFace,
    LeafFace,
    TransitFace,
}

#[derive(Debug)]
pub struct Face {
    cell_id: usize,

    face_index: FaceIndex,

    skip: bool,

    face_type: FaceType,

    sharp_feature: bool,

    feature_position: Vector3<f32>,

    twin: Option<Rc<RefCell<Face>>>,

    parent: Option<Rc<RefCell<Face>>>,

    /// consist with children faces
    children: Option<[Option<Rc<RefCell<Face>>>; 4]>,

    strips: Vec<Strip>,

    transit_segs: Vec<Vec<i8>>,
}

impl Face {
    pub fn new(face_index: FaceIndex, cell_id: usize) -> Self {
        Self {
            cell_id,
            face_index,
            skip: false,
            face_type: FaceType::LeafFace,
            sharp_feature: false,
            feature_position: Vector3::new(0.0, 0.0, 0.0),
            twin: None,
            parent: None,
            children: None,
            strips: Vec::new(),
            transit_segs: Vec::new(),
        }
    }
}

impl Face {
    pub fn set_twin(&mut self, twin: Rc<RefCell<Face>>) {
        self.twin = Some(twin);
    }

    pub fn get_twin(&self) -> &Option<Rc<RefCell<Face>>> {
        &self.twin
    }

    pub fn get_cell_id(&self) -> usize {
        self.cell_id
    }

    pub fn get_face_index(&self) -> FaceIndex {
        self.face_index
    }

    pub fn set_parent(&mut self, parent: Rc<RefCell<Face>>) {
        self.parent = Some(parent);
    }

    pub fn get_parent(&self) -> &Option<Rc<RefCell<Face>>> {
        &self.parent
    }

    pub fn get_children(&self) -> &Option<[Option<Rc<RefCell<Face>>>; 4]> {
        &self.children
    }

    pub fn set_child(&mut self, sub_face_index: SubFaceIndex, child: Rc<RefCell<Face>>) {
        self.children.get_or_insert([None, None, None, None])[sub_face_index as usize] =
            Some(child);
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

    pub fn set_skip(&mut self, skip: bool) {
        self.skip = skip;
    }

    pub fn get_skip(&self) -> bool {
        self.skip
    }

    pub fn set_sharp_feature(&mut self, sharp_feature: bool) {
        self.sharp_feature = sharp_feature;
    }

    pub fn get_sharp_feature(&self) -> bool {
        self.sharp_feature
    }

    pub fn set_feature_position(&mut self, feature_position: Vector3<f32>) {
        self.feature_position = feature_position;
    }

    pub fn get_feature_position(&self) -> Vector3<f32> {
        self.feature_position
    }

    pub fn set_transit_segs(&mut self, transit_segs: Vec<Vec<i8>>) {
        self.transit_segs = transit_segs;
    }

    pub fn get_transit_segs(&self) -> &Vec<Vec<i8>> {
        &self.transit_segs
    }
}
