use std::ops::{Add, Sub};

use bevy::prelude::{Component, Vec3};

use super::{TERRAIN_VOXEL_NUM_IN_CHUNK, TERRAIN_VOXEL_SIZE};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct VoxelGradedCoord {
    pub chunk_coord: TerrainChunkCoord,
    pub voxel_coord: TerrainVoxelCoord,
}

impl From<&TerrainGlobalCoord> for VoxelGradedCoord {
    fn from(value: &TerrainGlobalCoord) -> Self {
        Self {
            chunk_coord: TerrainChunkCoord::from(value),
            voxel_coord: TerrainVoxelCoord::from(value),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq, Component)]
pub struct TerrainGlobalCoord {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl From<&[i64; 3]> for TerrainGlobalCoord {
    fn from(value: &[i64; 3]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

impl TerrainGlobalCoord {
    pub fn from_location(location: &Vec3) -> TerrainGlobalCoord {
        let x = (location.x / TERRAIN_VOXEL_SIZE) as i64;
        let y = (location.y / TERRAIN_VOXEL_SIZE) as i64;
        let z = (location.z / TERRAIN_VOXEL_SIZE) as i64;

        Self { x, y, z }
    }

    pub fn from_local_coords(
        chunk_coord: &TerrainChunkCoord,
        voxel_coord: &TerrainVoxelCoord,
    ) -> Self {
        let x = chunk_coord.x * TERRAIN_VOXEL_NUM_IN_CHUNK as i64 + voxel_coord.x as i64;
        let y = chunk_coord.y * TERRAIN_VOXEL_NUM_IN_CHUNK as i64 + voxel_coord.y as i64;
        let z = chunk_coord.z * TERRAIN_VOXEL_NUM_IN_CHUNK as i64 + voxel_coord.z as i64;

        Self { x, y, z }
    }

    pub fn to_location(&self) -> Vec3 {
        Vec3::new(
            self.x as f32 * TERRAIN_VOXEL_SIZE,
            self.y as f32 * TERRAIN_VOXEL_SIZE,
            self.z as f32 * TERRAIN_VOXEL_SIZE,
        )
    }
}

#[derive(Debug, Default, Copy, Clone, Hash, Eq, PartialEq, Component)]
pub struct TerrainChunkCoord {
    pub x: i64,
    pub y: i64,
    pub z: i64,
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

/// global coord to chunk coord
impl From<&TerrainGlobalCoord> for TerrainChunkCoord {
    fn from(value: &TerrainGlobalCoord) -> Self {
        Self {
            x: value.x / TERRAIN_VOXEL_NUM_IN_CHUNK as i64,
            y: value.y / TERRAIN_VOXEL_NUM_IN_CHUNK as i64,
            z: value.z / TERRAIN_VOXEL_NUM_IN_CHUNK as i64,
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

/// section local coord
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct TerrainVoxelCoord {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}

impl From<&[usize; 3]> for TerrainVoxelCoord {
    fn from(value: &[usize; 3]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

/// from global voxel coord to local voxel coord
impl From<&TerrainGlobalCoord> for TerrainVoxelCoord {
    fn from(value: &TerrainGlobalCoord) -> Self {
        Self {
            x: (value.x % TERRAIN_VOXEL_NUM_IN_CHUNK as i64) as usize,
            y: (value.y % TERRAIN_VOXEL_NUM_IN_CHUNK as i64) as usize,
            z: (value.z % TERRAIN_VOXEL_NUM_IN_CHUNK as i64) as usize,
        }
    }
}
