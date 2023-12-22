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
