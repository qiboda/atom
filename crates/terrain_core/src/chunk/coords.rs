use std::ops::{Add, Sub};

use bevy::prelude::Component;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq, Component, Serialize, Deserialize)]
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

impl From<&[i64; 3]> for TerrainChunkCoord {
    fn from(value: &[i64; 3]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

impl Add<&TerrainChunkCoord> for &TerrainChunkCoord {
    type Output = TerrainChunkCoord;

    fn add(self, rhs: &TerrainChunkCoord) -> Self::Output {
        Self::Output {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub<&TerrainChunkCoord> for &TerrainChunkCoord {
    type Output = TerrainChunkCoord;

    fn sub(self, rhs: &TerrainChunkCoord) -> Self::Output {
        Self::Output {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
