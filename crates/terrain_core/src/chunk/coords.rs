use std::{
    fmt::{Display, Formatter},
    ops::{Add, Mul, Sub},
};

use bevy::{
    math::{Vec3, Vec3A},
    prelude::Component,
    reflect::Reflect,
};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Default, Copy, Clone, Hash, Eq, PartialEq, Component, Serialize, Deserialize, Reflect,
)]
pub struct TerrainChunkCoord {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl TerrainChunkCoord {
    pub fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    pub fn abs(&self) -> Self {
        Self {
            x: self.x.abs(),
            y: self.y.abs(),
            z: self.z.abs(),
        }
    }

    pub fn max_element(&self) -> i64 {
        self.x.max(self.y).max(self.z)
    }

    pub fn min_element(&self) -> i64 {
        self.x.min(self.y).min(self.z)
    }
}

impl TerrainChunkCoord {
    /// Chebyshev distance
    pub fn chebyshev_distance(&self) -> u64 {
        self.abs().max_element() as u64
    }

    /// Manhattan distance
    pub fn manhattan_distance(&self) -> u64 {
        self.abs().x as u64 + self.abs().y as u64 + self.abs().z as u64
    }
}

macro_rules! impl_from_terrain_chunk_coord {
    ($t: ty) => {
        impl From<[$t; 3]> for TerrainChunkCoord {
            fn from(value: [$t; 3]) -> Self {
                Self {
                    x: value[0] as i64,
                    y: value[1] as i64,
                    z: value[2] as i64,
                }
            }
        }
    };
}

impl_from_terrain_chunk_coord!(u64);
impl_from_terrain_chunk_coord!(i64);
impl_from_terrain_chunk_coord!(u32);
impl_from_terrain_chunk_coord!(i32);
impl_from_terrain_chunk_coord!(u16);
impl_from_terrain_chunk_coord!(i16);
impl_from_terrain_chunk_coord!(u8);
impl_from_terrain_chunk_coord!(i8);

impl From<Vec3> for TerrainChunkCoord {
    fn from(value: Vec3) -> Self {
        Self {
            x: value[0] as i64,
            y: value[1] as i64,
            z: value[2] as i64,
        }
    }
}

impl From<Vec3A> for TerrainChunkCoord {
    fn from(value: Vec3A) -> Self {
        Self {
            x: value[0] as i64,
            y: value[1] as i64,
            z: value[2] as i64,
        }
    }
}

impl Display for TerrainChunkCoord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

impl Add<TerrainChunkCoord> for TerrainChunkCoord {
    type Output = TerrainChunkCoord;

    fn add(self, rhs: TerrainChunkCoord) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub<TerrainChunkCoord> for TerrainChunkCoord {
    type Output = TerrainChunkCoord;

    fn sub(self, rhs: TerrainChunkCoord) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul<f32> for TerrainChunkCoord {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Self::Output {
        Self::Output {
            x: self.x as f32 * rhs,
            y: self.y as f32 * rhs,
            z: self.z as f32 * rhs,
        }
    }
}
