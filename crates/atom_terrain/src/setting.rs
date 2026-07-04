//! 地形全局配置。
//!
//! 定义 `TerrainSetting` 资源，控制 voxel 大小、chunk 粒度、地形范围与噪声种子。

use bevy::{prelude::*, render::extract_resource::ExtractResource};

/// 地形全局配置，主世界和渲染世界共享。
#[derive(Resource, Clone, Debug, ExtractResource)]
pub struct TerrainSetting {
    /// 单个 voxel 在世界空间的大小（米）
    pub voxel_size: f32,
    /// chunk 每条边的 voxel 数量（总 voxel = voxel_count³）
    pub voxel_count: u32,
    /// 整个地形的水平范围（米），用于 UV 和裁剪
    pub terrain_size: f32,
    /// 噪声种子
    pub seed: u32,
}

impl Default for TerrainSetting {
    fn default() -> Self {
        Self {
            voxel_size: 0.5,
            voxel_count: 30,
            terrain_size: 4096.0,
            seed: 42,
        }
    }
}

impl TerrainSetting {
    /// chunk 在世界空间的边长
    pub fn chunk_size(&self) -> f32 {
        self.voxel_size * self.voxel_count as f32
    }

    /// 密度场采样网格点数 = (voxel_count + 1)³
    pub fn grid_points_per_chunk(&self) -> u32 {
        let n = self.voxel_count + 1;
        n * n * n
    }
}
