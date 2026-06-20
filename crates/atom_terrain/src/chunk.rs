use bevy::{math::IVec3, prelude::*};
use std::collections::HashMap;

/// 地形 chunk 标记组件
#[derive(Component, Clone, Debug, Default)]
#[require(Transform, Visibility)]
pub struct TerrainChunk;

/// chunk 在网格中的整数坐标
#[derive(Component, Clone, Copy, Debug, Default, Hash, Eq, PartialEq)]
pub struct TerrainChunkCoord(pub IVec3);

impl TerrainChunkCoord {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    /// 世界坐标 → chunk 坐标
    pub fn from_world(world: Vec3, chunk_size: f32) -> Self {
        let div = |v: f32| (v / chunk_size).floor() as i32;
        Self(IVec3::new(div(world.x), div(world.y), div(world.z)))
    }

    /// chunk 左下角世界坐标
    pub fn to_world(&self, chunk_size: f32) -> Vec3 {
        self.0.as_vec3() * chunk_size
    }
}

/// chunk 加载/卸载消息
#[derive(Message, Clone, Debug)]
pub struct ChunkLoadMsg {
    pub coord: TerrainChunkCoord,
}

#[derive(Message, Clone, Debug)]
pub struct ChunkUnloadMsg {
    pub coord: TerrainChunkCoord,
}

/// 已加载 chunk 注册表
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
