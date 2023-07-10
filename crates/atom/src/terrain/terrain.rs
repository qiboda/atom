use bevy::{
    prelude::{Component, Entity},
    utils::HashMap,
};

use super::chunk::coords::TerrainChunkCoord;

#[derive(Debug, Component, Default)]
pub struct TerrainData {
    /// entity is TerrainChunk
    pub data: HashMap<TerrainChunkCoord, Entity>,
}

impl TerrainData {
    pub fn get_chunk_entity_by_coord(
        &self,
        terrain_chunk_coord: TerrainChunkCoord,
    ) -> Option<&Entity> {
        self.data.get(&terrain_chunk_coord)
    }

    pub fn new() -> TerrainData {
        Self::default()
    }
}
