use bevy::prelude::*;

use super::{strip::Strip, tables::FaceIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceType {
    BranchFace,
    LeafFace,    // only cell's face can be leaf face.
    TransitFace, // 如果两个相邻的cell的face不是同一级的，那么这个face就是transit face
}

#[derive(Debug)]
pub struct Face {
    face_index: FaceIndex,

    face_type: FaceType,

    // 一个face上的所有strip, 包括了transit face 和 leaf face
    strips: Vec<Strip>,

    // 缓存了所有子节点的顶点索引，仅仅在transit face上有用
    // 通过比较这里的顶点索引与strips的顶点索引，可以得到由于transit
    // face的存在，缺失的顶点索引。并链接到正确的位置。
    transit_segs: Vec<Vec<u32>>,
}

impl Face {
    pub fn new(face_index: FaceIndex, face_type: FaceType) -> Self {
        Self {
            face_index,
            face_type,
            strips: Vec::new(),
            transit_segs: Vec::new(),
        }
    }
}

impl Face {
    pub fn get_face_index(&self) -> FaceIndex {
        self.face_index
    }

    pub fn set_face_type(&mut self, face_type: FaceType) {
        self.face_type = face_type;
    }

    pub fn get_face_type(&self) -> &FaceType {
        &self.face_type
    }

    pub fn get_strips(&self) -> &Vec<Strip> {
        &self.strips
    }

    pub fn get_strips_mut(&mut self) -> &mut Vec<Strip> {
        &mut self.strips
    }

    pub fn set_transit_segs(&mut self, transit_segs: Vec<Vec<u32>>) {
        self.transit_segs = transit_segs;
    }

    pub fn get_transit_segs(&self) -> &Vec<Vec<u32>> {
        &self.transit_segs
    }
}

#[derive(Debug, Component)]
pub struct Faces {
    faces: [Face; 6],
}

impl Faces {
    pub fn new(face_type: FaceType) -> Self {
        Self {
            faces: [
                Face::new(FaceIndex::Back, face_type),
                Face::new(FaceIndex::Front, face_type),
                Face::new(FaceIndex::Bottom, face_type),
                Face::new(FaceIndex::Top, face_type),
                Face::new(FaceIndex::Left, face_type),
                Face::new(FaceIndex::Right, face_type),
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
