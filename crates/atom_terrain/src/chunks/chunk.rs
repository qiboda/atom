use bevy::prelude::*;

use std::ops::{Add, Sub};

use bevy::{
    math::{IVec3, Vec3},
    render::extract_component::ExtractComponent,
};

#[derive(Component, ExtractComponent, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
#[require(Transform, Visibility)]
pub struct TerrainChunk;

#[derive(
    Debug, Default, Copy, Clone, Hash, Eq, Reflect, PartialEq, Component, ExtractComponent,
)]
#[reflect(Component)]
#[require(TerrainChunk)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec3;

    #[test]
    fn test_new_and_accessors() {
        let coord = TerrainChunkCoord::new(1, -2, 3);
        assert_eq!(coord.x(), 1);
        assert_eq!(coord.y(), -2);
        assert_eq!(coord.z(), 3);
    }

    #[test]
    fn test_from_world_pos_origin() {
        let coord = TerrainChunkCoord::from_world_pos(Vec3::new(4.0, 0.0, 4.0), 8.0);
        assert_eq!(coord.x(), 0);
        assert_eq!(coord.y(), 0);
        assert_eq!(coord.z(), 0);
    }

    #[test]
    fn test_from_world_pos_positive() {
        let coord = TerrainChunkCoord::from_world_pos(Vec3::new(8.0, 8.0, 8.0), 8.0);
        assert_eq!(coord.x(), 1);
        assert_eq!(coord.y(), 1);
        assert_eq!(coord.z(), 1);
    }

    #[test]
    fn test_from_world_pos_negative() {
        let coord = TerrainChunkCoord::from_world_pos(Vec3::new(-1.0, -1.0, -1.0), 8.0);
        assert_eq!(coord.x(), -1);
        assert_eq!(coord.y(), -1);
        assert_eq!(coord.z(), -1);
    }

    #[test]
    fn test_to_world_pos() {
        let coord = TerrainChunkCoord::new(2, -1, 3);
        let world = coord.to_world_pos(8.0);
        assert_eq!(world.x, 16.0);
        assert_eq!(world.y, -8.0);
        assert_eq!(world.z, 24.0);
    }

    #[test]
    fn test_chebyshev_distance() {
        let a = TerrainChunkCoord::new(0, 0, 0);
        let b = TerrainChunkCoord::new(3, 1, 2);
        assert_eq!(a.chebyshev_distance(&b), 3);
        assert_eq!(a.chebyshev_distance(&a), 0);
    }

    #[test]
    fn test_chebyshev_distance_xz() {
        let a = TerrainChunkCoord::new(0, 0, 0);
        let b = TerrainChunkCoord::new(3, 100, 2);
        assert_eq!(a.chebyshev_distance_xz(&b), 3); // ignores y
    }

    #[test]
    fn test_add_sub() {
        let a = TerrainChunkCoord::new(1, 2, 3);
        let b = TerrainChunkCoord::new(4, 5, 6);
        let sum = a + b;
        assert_eq!(sum.x(), 5);
        assert_eq!(sum.y(), 7);
        assert_eq!(sum.z(), 9);

        let diff = b - a;
        assert_eq!(diff.x(), 3);
        assert_eq!(diff.y(), 3);
        assert_eq!(diff.z(), 3);
    }

    #[test]
    fn test_as_array_and_ivec3() {
        let coord = TerrainChunkCoord::new(1, 2, 3);
        assert_eq!(coord.as_array_i32(), [1, 2, 3]);
        assert_eq!(coord.as_ivec3(), IVec3::new(1, 2, 3));
    }

    #[test]
    fn test_from_array() {
        let arr = [7, 8, 9];
        let coord = TerrainChunkCoord::from(&arr);
        assert_eq!(coord.x(), 7);
        assert_eq!(coord.y(), 8);
        assert_eq!(coord.z(), 9);
    }
}
