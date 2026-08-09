//! 地形 chunk 组件与加载同步。
//!
//! 定义 chunk 标记组件、坐标系统、加载/卸载消息以及已加载 chunk 注册表。

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
    /// 从 (x, y, z) 整数坐标创建
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

/// chunk 加载消息
#[derive(Message, Clone, Debug)]
pub struct ChunkLoadMsg {
    /// 要加载的 chunk 坐标
    pub coord: TerrainChunkCoord,
}

/// chunk 卸载消息
#[derive(Message, Clone, Debug)]
pub struct ChunkUnloadMsg {
    /// 要卸载的 chunk 坐标
    pub coord: TerrainChunkCoord,
}

/// 已加载 chunk 注册表
#[derive(Resource, Default, Debug)]
pub struct TerrainLoadedChunks {
    chunks: HashMap<TerrainChunkCoord, Entity>,
}

impl TerrainLoadedChunks {
    /// 注册一个新的 chunk
    pub fn insert(&mut self, coord: TerrainChunkCoord, entity: Entity) {
        self.chunks.insert(coord, entity);
    }

    /// 移除并返回指定坐标的 chunk
    pub fn remove(&mut self, coord: &TerrainChunkCoord) -> Option<Entity> {
        self.chunks.remove(coord)
    }

    /// 检查指定坐标的 chunk 是否已加载
    pub fn contains(&self, coord: &TerrainChunkCoord) -> bool {
        self.chunks.contains_key(coord)
    }

    /// 获取指定坐标的 chunk 实体
    pub fn get(&self, coord: &TerrainChunkCoord) -> Option<Entity> {
        self.chunks.get(coord).copied()
    }

    /// 遍历所有已加载 chunk
    pub fn iter(&self) -> impl Iterator<Item = (&TerrainChunkCoord, &Entity)> {
        self.chunks.iter()
    }

    /// 已加载 chunk 数量
    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    /// 是否没有任何已加载 chunk
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coord(x: i32, y: i32, z: i32) -> TerrainChunkCoord {
        TerrainChunkCoord::new(x, y, z)
    }

    // ── TerrainChunkCoord ──

    #[test]
    fn new_stores_ivec3() {
        assert_eq!(coord(1, -2, 3).0, IVec3::new(1, -2, 3));
        assert_eq!(TerrainChunkCoord::default().0, IVec3::ZERO);
    }

    #[test]
    fn from_world_floors_world_over_chunk_size() {
        let cs = 15.0;
        assert_eq!(
            TerrainChunkCoord::from_world(Vec3::new(15.0, 0.0, 15.0), cs),
            coord(1, 0, 1)
        );
        assert_eq!(
            TerrainChunkCoord::from_world(Vec3::new(14.999, -0.001, 0.0), cs),
            coord(0, -1, 0)
        );
        // 负坐标边界：-0.001 → floor(-6.7e-5) = -1
        assert_eq!(
            TerrainChunkCoord::from_world(Vec3::new(-0.001, 0.0, 0.0), cs),
            coord(-1, 0, 0)
        );
        assert_eq!(
            TerrainChunkCoord::from_world(Vec3::new(-15.0, 0.0, 0.0), cs),
            coord(-1, 0, 0)
        );
        assert_eq!(
            TerrainChunkCoord::from_world(Vec3::new(-15.001, 0.0, 0.0), cs),
            coord(-2, 0, 0)
        );
    }

    #[test]
    fn to_world_scales_coord_by_chunk_size() {
        assert_eq!(coord(2, -1, 3).to_world(15.0), Vec3::new(30.0, -15.0, 45.0));
        assert_eq!(coord(-2, 0, 1).to_world(10.0), Vec3::new(-20.0, 0.0, 10.0));
        assert_eq!(coord(0, 0, 0).to_world(15.0), Vec3::ZERO);
    }

    #[test]
    fn world_round_trip() {
        for &(x, y, z) in &[
            (0, 0, 0),
            (3, -2, 7),
            (-5, 1, -9),
            (100000, -65536, 7),
            (-12345, 0, 99999),
        ] {
            let c = coord(x, y, z);
            assert_eq!(TerrainChunkCoord::from_world(c.to_world(15.0), 15.0), c);
        }
    }

    #[test]
    fn from_world_zero_chunk_size_saturates() {
        // 除零退化输入：f32 → i32 饱和转换，不 panic
        let p = TerrainChunkCoord::from_world(Vec3::new(1.0, 0.0, 1.0), 0.0);
        assert_eq!(p.0.x, i32::MAX);
        assert_eq!(p.0.z, i32::MAX);
        // 0/0 = NaN → 转换为 0
        let z = TerrainChunkCoord::from_world(Vec3::ZERO, 0.0);
        assert_eq!(z.0, IVec3::ZERO);
    }

    // ── TerrainLoadedChunks ──

    #[test]
    fn loaded_chunks_insert_get_contains_len() {
        let mut chunks = TerrainLoadedChunks::default();
        assert!(chunks.is_empty());
        assert_eq!(chunks.len(), 0);

        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);
        chunks.insert(coord(0, 0, 0), e1);
        chunks.insert(coord(1, 0, 0), e2);

        assert_eq!(chunks.len(), 2);
        assert!(!chunks.is_empty());
        assert!(chunks.contains(&coord(0, 0, 0)));
        assert!(!chunks.contains(&coord(2, 0, 0)));
        assert_eq!(chunks.get(&coord(0, 0, 0)), Some(e1));
        assert_eq!(chunks.get(&coord(5, 5, 5)), None);

        // 同坐标重复 insert 覆盖 entity，数量不变
        chunks.insert(coord(0, 0, 0), e2);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks.get(&coord(0, 0, 0)), Some(e2));
    }

    #[test]
    fn loaded_chunks_remove() {
        let mut chunks = TerrainLoadedChunks::default();
        assert_eq!(chunks.remove(&coord(0, 0, 0)), None);

        chunks.insert(coord(0, 0, 0), Entity::from_bits(1));
        chunks.insert(coord(1, 0, 0), Entity::from_bits(2));
        chunks.insert(coord(2, 0, 0), Entity::from_bits(3));

        assert_eq!(chunks.remove(&coord(1, 0, 0)), Some(Entity::from_bits(2)));
        assert_eq!(chunks.len(), 2);
        assert!(!chunks.contains(&coord(1, 0, 0)));
        // 再次移除返回 None
        assert_eq!(chunks.remove(&coord(1, 0, 0)), None);
    }

    #[test]
    fn loaded_chunks_iter() {
        let mut chunks = TerrainLoadedChunks::default();
        assert_eq!(chunks.iter().count(), 0);

        let keys = [coord(0, 0, 0), coord(1, 0, 0), coord(-2, 3, 4)];
        for (i, k) in keys.iter().enumerate() {
            chunks.insert(*k, Entity::from_bits(i as u64 + 1));
        }
        let mut seen: Vec<TerrainChunkCoord> = chunks.iter().map(|(&k, _)| k).collect();
        seen.sort_by_key(|c| (c.0.x, c.0.y, c.0.z));
        let mut expected: Vec<TerrainChunkCoord> = keys.to_vec();
        expected.sort_by_key(|c| (c.0.x, c.0.y, c.0.z));
        assert_eq!(seen, expected);
    }
}
