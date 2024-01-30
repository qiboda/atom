use bevy::prelude::*;
use strum::EnumCount;
use strum_macros::{EnumIter, FromRepr};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter, EnumCount, FromRepr)]
pub enum FaceIndex {
    Back = 0,
    Front = 1,
    Bottom = 2,
    Top = 3,
    Left = 4,
    Right = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceType {
    BranchFace,
    LeafFace,    // only cell's face can be leaf face.
    TransitFace, // 如果两个相邻的cell的face不是同一级的，那么这个face就是transit face
}

#[derive(Debug)]
pub struct Face {
    pub face_index: FaceIndex,

    pub face_type: FaceType,
}

impl Face {
    pub fn new(face_index: FaceIndex, face_type: FaceType) -> Self {
        Self {
            face_index,
            face_type,
        }
    }
}

#[derive(Debug, Component)]
pub struct Faces {
    pub faces: [Face; FaceIndex::COUNT],
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
