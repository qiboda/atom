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

#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct TerrainSizeSetting {
    // TODO: 是否限制最大水平范围
    // 地形高度范围，单位为chunk数量
    pub height_range: RangeInclusive<i32>,
}

impl Default for TerrainSetting {
    fn default() -> Self {
        Self {
            qef_solver: true,
            qef_solver_threshold: 0.1,
            qef_stddev: 0.1,
            chunk_setting: TerrainChunkSetting {
                voxel_size: 1.0,
                voxel_count: 16,
            },
            size_setting: TerrainSizeSetting {
                height_range: -8..=16,
            },
        }
    }
}

impl TerrainSetting {
    pub fn get_voxel_count_in_chunk(&self) -> u32 {
        self.chunk_setting.voxel_count as u32
    }

    pub fn get_voxel_count_in_compute(&self) -> u32 {
        self.chunk_setting.voxel_count as u32 + 1
    }

    pub fn is_in_height_range(&self, height: i32) -> bool {
        self.size_setting.height_range.contains(&height)
    }

    pub fn get_voxel_size(&self) -> f32 {
        self.chunk_setting.voxel_size
    }

    pub fn get_chunk_size(&self) -> f32 {
        self.get_voxel_size() * self.get_voxel_count_in_chunk() as f32
    }
}
