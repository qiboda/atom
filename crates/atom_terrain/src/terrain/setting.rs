use std::ops::RangeInclusive;

use bevy::{prelude::*, render::extract_resource::ExtractResource};
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect, ExtractResource)]
#[reflect(Resource)]
pub struct TerrainSetting {
    /// chunk 设置
    pub chunk_setting: TerrainChunkSetting,

    pub size_setting: TerrainSizeSetting,

    /// 是否启用octree的节点收缩
    pub qef_solver: bool,
    /// octree的深度对应的qef的阈值，小于这个阈值，则可以收缩节点。
    pub qef_solver_threshold: f32,
    /// qef solver的单位标准差
    pub qef_stddev: f32,
}

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct TerrainChunkSetting {
    pub voxel_size: f32,
    /**
     * 单个chunk在每个维度上的体素数量
     */
    pub voxel_count: u8,
}

impl TerrainChunkSetting {
    /**
     * 单个chunk的大小
     */
    pub fn get_chunk_size(&self) -> f32 {
        self.voxel_size * self.voxel_count as f32
    }
}

impl Default for TerrainChunkSetting {
    fn default() -> Self {
        TerrainChunkSetting {
            voxel_size: 0.5,
            voxel_count: 16,
        }
    }
}
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct TerrainSizeSetting {
    // 地形高度范围，单位为chunk数量
    pub height_range: RangeInclusive<i32>,
    // 地形水平范围，单位为chunk数量
    pub horizontal_range: RangeInclusive<i32>,
}

impl Default for TerrainSizeSetting {
    fn default() -> Self {
        Self {
            height_range: -8..=16,
            horizontal_range: 0..=512,
        }
    }
}

impl Default for TerrainSetting {
    fn default() -> Self {
        Self {
            qef_solver: true,
            qef_solver_threshold: 0.1,
            qef_stddev: 0.1,
            chunk_setting: Default::default(),
            size_setting: Default::default(),
        }
    }
}

impl TerrainSetting {
    /**
     * 获取单个chunk中的体素数量
     */
    pub fn get_voxel_count_in_chunk(&self) -> u32 {
        self.chunk_setting.voxel_count as u32
    }

    /**
     * 获取用于计算的体素数量（比实际多1个，用于计算Chunk的缝合边界)
     */
    pub fn get_voxel_count_in_compute(&self) -> u32 {
        self.chunk_setting.voxel_count as u32 + 1
    }

    /**
     * 获取地形的高度范围，单位为世界单位（米）
     */
    pub fn get_height_range_size(&self) -> RangeInclusive<f32> {
        let chunk_size = self.get_chunk_size();
        *self.size_setting.height_range.start() as f32 * chunk_size
            ..=*self.size_setting.height_range.end() as f32 * chunk_size
    }

    /**
     * 检查给定的高度是否在地形的高度范围内, 单位为chunk数量
     */
    pub fn is_in_height_range(&self, height: i32) -> bool {
        self.size_setting.height_range.contains(&height)
    }

    /**
     * 获取单个体素的大小，单位为世界单位（米）
     */
    pub fn get_voxel_size(&self) -> f32 {
        self.chunk_setting.voxel_size
    }

    /**
     * 获取单个chunk的大小，单位为世界单位（米）
     */
    pub fn get_chunk_size(&self) -> f32 {
        self.get_voxel_size() * self.get_voxel_count_in_chunk() as f32
    }

    /**
     * 获取地形的水平尺寸，单位为世界单位（米）
     */
    pub fn get_terrain_size(&self) -> f32 {
        let horizontal_range = &self.size_setting.horizontal_range;
        let horizontal_size = horizontal_range.end() - horizontal_range.start() + 1;
        horizontal_size as f32 * self.get_chunk_size()
    }
}
