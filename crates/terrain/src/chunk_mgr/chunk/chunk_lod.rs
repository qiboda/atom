use std::ops::{Deref, DerefMut};

use bevy::{
    math::{bounding::Aabb3d, Vec3A},
    prelude::Component,
    reflect::Reflect,
};

pub type LodType = u8;
pub type OctreeDepthType = LodType;

#[derive(Debug, Component, Reflect, PartialEq, Eq, Clone, Copy)]
pub struct TerrainChunkLod {
    lod: LodType,
}

impl Default for TerrainChunkLod {
    fn default() -> Self {
        Self::new(LodType::MAX)
    }
}

impl TerrainChunkLod {
    pub fn new(lod: LodType) -> Self {
        Self { lod }
    }

    pub fn set_lod(&mut self, lod: LodType) -> bool {
        if self.lod == lod {
            return false;
        }
        self.lod = lod;
        true
    }

    pub fn get_lod(&self) -> LodType {
        self.lod
    }
}

#[derive(Debug, Component, Reflect, Clone)]
pub struct TerrainChunkAabb(pub Aabb3d);

impl Default for TerrainChunkAabb {
    fn default() -> Self {
        Self(Aabb3d::new(Vec3A::ZERO, Vec3A::ZERO))
    }
}

impl Deref for TerrainChunkAabb {
    type Target = Aabb3d;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TerrainChunkAabb {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
