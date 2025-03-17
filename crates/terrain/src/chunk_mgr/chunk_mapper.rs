use crate::chunk_mgr::chunk::comp::TerrainChunkAddress;
use bevy::{prelude::*, render::extract_resource::ExtractResource, utils::HashMap};

// TODO: 需要对应的渲染世界的entity。
#[derive(Debug, Resource, Default, ExtractResource, Clone)]
pub struct TerrainChunkMapper {
    /// entity is TerrainChunk, and in main world
    pub data: HashMap<TerrainChunkAddress, Entity>,
}

impl TerrainChunkMapper {
    pub fn get_chunk_entity(&self, terrain_chunk_address: TerrainChunkAddress) -> Option<&Entity> {
        self.data.get(&terrain_chunk_address)
    }

    pub fn new() -> TerrainChunkMapper {
        Self::default()
    }
}
