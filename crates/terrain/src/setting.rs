use std::ops::RangeInclusive;

use bevy::{prelude::*, render::extract_resource::ExtractResource};
use serde::{Deserialize, Serialize};
use settings::{Setting, SettingValidate};

use crate::lod::{lod_octree::LodOctreeDepthType, morton_code::MortonCode};

#[derive(
    Setting, Resource, Debug, Clone, Serialize, Deserialize, TypePath, Asset, ExtractResource,
)]
pub struct TerrainSetting {
    /// chunk大小
    pub chunk_size: f32,
    /// chunk octree的深度
    pub chunk_depth: u8,
    /// 是否启用octree的节点收缩
    pub qef_solver: bool,
    /// octree的深度对应的qef的阈值，小于这个阈值，则可以收缩节点。
    pub qef_solver_threshold: f32,
    /// qef solver的单位标准差
    pub qef_stddev: f32,
    /// lod octree depth
    pub lod_octree_depth: LodOctreeDepthType,
    /// 地形的最远可见距离是否和相机的远裁剪面一致
    pub camera_far_limit: bool,
    /// 地形的基础可见范围
    pub base_visibility_range: f32,
    /// 地形高度的范围
    pub terrain_height_range: RangeInclusive<f32>,
}

impl SettingValidate for TerrainSetting {
    fn validate(&self) -> bool {
        let mut validation = true;
        let log_2_size = self.chunk_size.log2();
        if log_2_size.fract() != 0.0 {
            error!("chunk_size must be 2^n");
            validation = false;
        }

        let max_depth = MortonCode::MAX_LEVEL;
        if self.lod_octree_depth > max_depth {
            error!(
                "lod_octree_depth value is invalid, must be in [0, {}]",
                max_depth
            );
            validation = false;
        }

        validation
    }
}

impl Default for TerrainSetting {
    fn default() -> Self {
        Self {
            chunk_size: 16.0,
            chunk_depth: 5,
            qef_solver: true,
            qef_solver_threshold: 0.1,
            qef_stddev: 0.1,
            lod_octree_depth: 9,
            camera_far_limit: true,
            base_visibility_range: 256.0,
            terrain_height_range: -128.0..=256.0,
        }
    }
}

impl TerrainSetting {
    pub fn get_terrain_size(&self) -> f32 {
        self.get_chunk_size(0)
    }

    pub fn get_lod_octree_depth(&self) -> LodOctreeDepthType {
        self.lod_octree_depth
    }

    pub fn get_chunk_size(&self, depth: LodOctreeDepthType) -> f32 {
        assert!(self.lod_octree_depth >= depth);
        let lod = self.lod_octree_depth - depth;
        self.chunk_size * 2.0f32.powi(lod as i32)
    }

    pub fn get_voxel_num_in_chunk(&self) -> usize {
        1 << self.chunk_depth
    }

    pub fn get_default_voxel_size(&self) -> f32 {
        self.chunk_size / self.get_voxel_num_in_chunk() as f32
    }

    pub fn get_voxel_size(&self, depth: LodOctreeDepthType) -> f32 {
        assert!(self.lod_octree_depth >= depth);
        let lod = self.lod_octree_depth - depth;
        self.get_default_voxel_size() * 2.0f32.powi(lod as i32)
    }

    pub fn is_in_height_range(&self, height: f32) -> bool {
        self.terrain_height_range.contains(&height)
    }

    pub fn get_terrain_max_height(&self) -> f32 {
        *self.terrain_height_range.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_chunk() {
        let setting = TerrainSetting {
            chunk_size: 64.0,
            chunk_depth: 7,
            qef_solver: true,
            qef_solver_threshold: 0.1,
            qef_stddev: 0.1,
            lod_octree_depth: 4,
            camera_far_limit: true,
            ..default()
        };
        assert_eq!(setting.get_chunk_size(4), 64.0);
        assert_eq!(setting.get_chunk_size(3), 128.0);
        assert_eq!(setting.get_chunk_size(2), 256.0);
        assert_eq!(setting.get_chunk_size(1), 512.0);
        assert_eq!(setting.get_chunk_size(0), 1024.0);
    }

    #[test]
    fn test_terrain_voxel() {
        let setting = TerrainSetting {
            chunk_size: 64.0,
            chunk_depth: 7,
            qef_solver: true,
            qef_solver_threshold: 0.1,
            qef_stddev: 0.1,
            lod_octree_depth: 4,
            camera_far_limit: true,
            ..default()
        };
        assert_eq!(setting.get_default_voxel_size(), 0.5);
        assert_eq!(setting.get_voxel_num_in_chunk(), 128);

        assert_eq!(setting.get_voxel_size(4), 0.5);
        assert_eq!(setting.get_voxel_size(3), 1.0);
        assert_eq!(setting.get_voxel_size(2), 2.0);
        assert_eq!(setting.get_voxel_size(1), 4.0);
        assert_eq!(setting.get_voxel_size(0), 8.0);
    }

    #[test]
    #[should_panic]
    fn test_terrain_chunk_crash() {
        let setting = TerrainSetting {
            chunk_size: 64.0,
            chunk_depth: 7,
            qef_solver: true,
            qef_solver_threshold: 0.1,
            qef_stddev: 0.1,
            lod_octree_depth: 4,
            camera_far_limit: true,
            ..default()
        };
        setting.get_chunk_size(5);
    }

    #[test]
    #[should_panic]
    fn test_terrain_voxel_crash() {
        let setting = TerrainSetting {
            chunk_size: 64.0,
            chunk_depth: 7,
            qef_solver: true,
            qef_solver_threshold: 0.1,
            qef_stddev: 0.1,
            lod_octree_depth: 4,
            camera_far_limit: true,
            ..default()
        };
        setting.get_voxel_size(5);
    }

    #[test]
    fn test_terrain_size() {
        let setting = TerrainSetting {
            chunk_size: 64.0,
            chunk_depth: 7,
            qef_solver: true,
            qef_solver_threshold: 0.1,
            qef_stddev: 0.1,
            lod_octree_depth: 8,
            camera_far_limit: true,
            ..default()
        };
        assert_eq!(setting.get_terrain_size(), 16384.0);
    }
}
