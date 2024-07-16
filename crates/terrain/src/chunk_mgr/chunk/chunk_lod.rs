use bevy::{prelude::Component, reflect::Reflect};

pub type LodType = u8;

#[derive(Debug, Component, Reflect, PartialEq, Eq, Clone, Copy)]
pub struct TerrainChunkLod {
    old_lod: LodType,
    new_lod: LodType,
}

impl Default for TerrainChunkLod {
    fn default() -> Self {
        Self::new()
    }
}

impl TerrainChunkLod {
    pub fn new() -> Self {
        Self {
            old_lod: LodType::MAX,
            new_lod: LodType::MAX,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.old_lod != self.new_lod
    }

    pub fn clean_dirty(&mut self) {
        self.old_lod = self.new_lod;
    }

    pub fn set_lod(&mut self, lod: LodType) {
        if self.new_lod == lod {
            return;
        }
        self.old_lod = self.new_lod;
        self.new_lod = lod;
    }

    pub fn get_lod(&self) -> LodType {
        self.new_lod
    }

    pub fn get_old_lod(&self) -> LodType {
        self.old_lod
    }
}
