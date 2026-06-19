use bevy::{prelude::*, render::extract_component::ExtractComponent};

#[derive(
    Component, Reflect, ExtractComponent, Default, Hash, Debug, Clone, Copy, PartialEq, Eq,
)]
#[reflect(Component)]
pub enum TerrainChunkMeshingState {
    #[default]
    Idle,
    Meshing,
}

/// 当前 chunk 的 LOD 级别 (0=最高精度, 2=最低精度)
#[derive(Component, Reflect, ExtractComponent, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[reflect(Component)]
pub struct TerrainChunkLod(pub u32);

impl TerrainChunkLod {
    /// 根据世界坐标距离计算 LOD 级别
    /// - 0-64m: LOD 0 (16³, 0.5m voxel)
    /// - 64-128m: LOD 1 (8³, 1.0m voxel)
    /// - 128-256m: LOD 2 (4³, 2.0m voxel)
    pub fn from_distance(distance: f32) -> Self {
        if distance < 64.0 {
            TerrainChunkLod(0)
        } else if distance < 128.0 {
            TerrainChunkLod(1)
        } else {
            TerrainChunkLod(2)
        }
    }

    /// 获取此 LOD 级别的体素数量和大小
    pub fn voxel_config(&self) -> (u32, f32) {
        match self.0 {
            0 => (16, 0.5),
            1 => (8, 1.0),
            2 => (4, 2.0),
            _ => (16, 0.5),
        }
    }
}
