use bevy::{prelude::Component, reflect::Reflect};

pub type LodType = u8;

#[derive(Debug, Component, Reflect, PartialEq, Eq, Clone, Copy)]
pub struct TerrainChunkLod {
    lod: LodType,
}

impl Default for TerrainChunkLod {
    fn default() -> Self {
        Self::new()
    }
}

impl TerrainChunkLod {
    pub fn new() -> Self {
        Self { lod: LodType::MAX }
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
