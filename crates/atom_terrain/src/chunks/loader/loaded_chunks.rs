use bevy::{platform::collections::HashMap, prelude::*};

use crate::chunks::chunk::TerrainChunkCoord;

/// Chunk 加载请求消息
#[derive(Message, Debug, Clone)]
pub struct TerrainChunkLoadMsg {
    pub coord: TerrainChunkCoord,
}

/// Chunk 卸载请求消息
#[derive(Message, Debug, Clone)]
pub struct TerrainChunkUnloadMsg {
    pub coord: TerrainChunkCoord,
}

/// 追踪已加载的 chunks
#[derive(Resource, Default, Debug)]
pub struct TerrainLoadedChunks {
    chunks: HashMap<TerrainChunkCoord, Entity>,
}

impl TerrainLoadedChunks {
    pub fn insert(&mut self, coord: TerrainChunkCoord, entity: Entity) {
        self.chunks.insert(coord, entity);
    }

    pub fn remove(&mut self, coord: &TerrainChunkCoord) -> Option<Entity> {
        self.chunks.remove(coord)
    }

    pub fn contains(&self, coord: &TerrainChunkCoord) -> bool {
        self.chunks.contains_key(coord)
    }

    pub fn get(&self, coord: &TerrainChunkCoord) -> Option<Entity> {
        self.chunks.get(coord).copied()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&TerrainChunkCoord, &Entity)> {
        self.chunks.iter()
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}
