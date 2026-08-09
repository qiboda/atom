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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-5,
            "expected {a} ≈ {b} but diff = {}",
            (a - b).abs()
        );
    }

    #[test]
    fn default_values() {
        let s = TerrainSetting::default();
        assert_approx(s.voxel_size, 0.5);
        assert_eq!(s.voxel_count, 30);
        assert_approx(s.terrain_size, 4096.0);
        assert_eq!(s.seed, 42);
    }

    #[test]
    fn default_chunk_size() {
        assert_approx(TerrainSetting::default().chunk_size(), 15.0);
    }

    #[test]
    fn default_grid_points_per_chunk() {
        assert_eq!(
            TerrainSetting::default().grid_points_per_chunk(),
            31u32.pow(3)
        );
    }

    #[test]
    fn custom_setting_getters() {
        let s = TerrainSetting {
            voxel_size: 2.0,
            voxel_count: 10,
            terrain_size: 1000.0,
            seed: 7,
        };
        assert_approx(s.chunk_size(), 20.0);
        assert_eq!(s.grid_points_per_chunk(), 11u32.pow(3));
    }

    #[test]
    fn grid_points_single_voxel() {
        let s = TerrainSetting {
            voxel_count: 1,
            ..Default::default()
        };
        assert_eq!(s.grid_points_per_chunk(), 8);
    }

    #[test]
    fn grid_points_zero_voxels() {
        let s = TerrainSetting {
            voxel_count: 0,
            ..Default::default()
        };
        assert_eq!(s.grid_points_per_chunk(), 1);
    }

    #[test]
    fn chunk_size_zero_voxel_size() {
        let s = TerrainSetting {
            voxel_size: 0.0,
            ..Default::default()
        };
        assert_approx(s.chunk_size(), 0.0);
    }
}
