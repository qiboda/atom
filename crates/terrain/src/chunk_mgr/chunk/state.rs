use bevy::prelude::*;

use super::chunk_lod::LodType;

#[derive(Debug, Component)]
pub enum TerrainChunkState {
    CreateMainMesh(LodType),
    CreateMainMeshOver,
    // 左右是轴的正方向。
    CreateSeamMesh(LodType, LodType),
    Done,
}
