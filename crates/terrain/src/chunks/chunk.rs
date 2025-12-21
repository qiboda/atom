use bevy::prelude::*;

use std::ops::{Add, Sub};

use bevy::{
    math::{IVec3, Vec3},
    render::extract_component::ExtractComponent,
};

#[derive(Component, Debug, Default, Reflect)]
#[require(Transform, Visibility)]
pub struct TerrainChunk;

#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq, Component)]
pub struct TerrainChunkLod {
    pub lod: u8,
}

#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq, Component, ExtractComponent)]
pub struct TerrainChunkCoord(IVec3);

impl TerrainChunkCoord {
    /// Create from signed 64-bit coords; will cast to `i32` for `IVec3`.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self(IVec3::new(x, y, z))
    }

    pub fn x(&self) -> i32 {
        self.0.x
    }
    pub fn y(&self) -> i32 {
        self.0.y
    }
    pub fn z(&self) -> i32 {
        self.0.z
    }

    /// 从世界坐标创建 chunk 坐标
    pub fn from_world_pos(world_pos: Vec3, chunk_size: f32) -> Self {
        let x = (world_pos.x / chunk_size).floor() as i32;
        let y = (world_pos.y / chunk_size).floor() as i32;
        let z = (world_pos.z / chunk_size).floor() as i32;
        Self::new(x, y, z)
    }

    pub fn to_world_pos(&self, chunk_size: f32) -> Vec3 {
        Vec3::new(
            self.0.x as f32 * chunk_size,
            self.0.y as f32 * chunk_size,
            self.0.z as f32 * chunk_size,
        )
    }

    pub fn as_array_i32(&self) -> [i32; 3] {
        let v = self.0;
        [v.x, v.y, v.z]
    }

    pub fn as_ivec3(&self) -> IVec3 {
        self.0
    }

    // 获取切比雪夫距离
    pub fn chebyshev_distance(&self, other: &TerrainChunkCoord) -> i32 {
        let delta = self.0 - other.0;
        delta.x.abs().max(delta.y.abs()).max(delta.z.abs())
    }

    // 获取切比雪夫距离, 仅XZ平面
    pub fn chebyshev_distance_xz(&self, other: &TerrainChunkCoord) -> i32 {
        let delta = self.0 - other.0;
        delta.x.abs().max(delta.z.abs())
    }

    /**
     * LOD 级别提升，精度下降
     */
    pub fn lod_bias_up(&self, lod: u8) -> TerrainChunkCoord {
        let factor = 2i32.pow(lod as u32);
        TerrainChunkCoord(IVec3::new(
            (self.0.x as f32 / factor as f32).floor() as i32,
            (self.0.y as f32 / factor as f32).floor() as i32,
            (self.0.z as f32 / factor as f32).floor() as i32,
        ))
    }

    /**
     * LOD 级别降低，精度上升
     */
    pub fn lod_bias_down(&self, lod: u8) -> TerrainChunkCoord {
        let factor = 2i32.pow(lod as u32);
        TerrainChunkCoord(IVec3::new(
            self.0.x * factor,
            self.0.y * factor,
            self.0.z * factor,
        ))
    }
}

impl From<&[i32; 3]> for TerrainChunkCoord {
    fn from(value: &[i32; 3]) -> Self {
        Self(IVec3::new(value[0], value[1], value[2]))
    }
}

impl Add for TerrainChunkCoord {
    type Output = TerrainChunkCoord;

    fn add(self, rhs: TerrainChunkCoord) -> Self::Output {
        TerrainChunkCoord(self.0 + rhs.0)
    }
}

impl Sub for TerrainChunkCoord {
    type Output = TerrainChunkCoord;

    fn sub(self, rhs: TerrainChunkCoord) -> Self::Output {
        TerrainChunkCoord(self.0 - rhs.0)
    }
}
