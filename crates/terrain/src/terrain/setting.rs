use std::ops::RangeInclusive;

use bevy::{prelude::*, render::extract_resource::ExtractResource};
use serde::{Deserialize, Serialize};

#[derive(Resource, Debug, Clone, Serialize, Deserialize, Reflect, ExtractResource)]
pub struct TerrainSetting {
    /// chunk 设置
    pub chunk_setting: TerrainChunkSetting,

    pub size_setting: TerrainSizeSetting,

    /// Clipmap 配置
    pub clipmap_config: ClipmapConfig,

    /// 是否启用octree的节点收缩
    pub qef_solver: bool,
    /// octree的深度对应的qef的阈值，小于这个阈值，则可以收缩节点。
    pub qef_solver_threshold: f32,
    /// qef solver的单位标准差
    pub qef_stddev: f32,

    pub stitch_seam_scheme: StitchSeamScheme,
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
    // 地形高度范围，单位为chunk数量
    pub height_range: RangeInclusive<i32>,
}

/// Clipmap 配置，只负责水平方向的 LOD 管理
#[derive(Debug, Clone, Reflect, Serialize, Deserialize)]
pub struct ClipmapConfig {
    /// LOD 级别数量（0 是最高细节）
    pub lod_count: u8,
    /// LOD0 级别的半径，单位是 chunk 数量，之后每增加一个 LOD 级别，半径翻倍
    pub lod0_radius: u8,
}

impl Default for ClipmapConfig {
    fn default() -> Self {
        Self {
            lod_count: 4,
            lod0_radius: 4,
        }
    }
}

impl ClipmapConfig {
    /// 获取指定 LOD 级别的半径，单位是 chunk 数量
    pub fn get_radius_by_lod(&self, lod: u8) -> u8 {
        self.lod0_radius * 3u8.pow(lod as u32)
    }
}

#[derive(Eq, PartialEq, Debug, Reflect, Clone, Serialize, Deserialize)]
pub enum StitchSeamScheme {
    DualContouring,
    NeighborConnect,
}

// impl SettingValidate for TerrainSetting {
//     fn validate(&self) -> bool {
//         let mut validation = true;
//         let log_2_size = self.chunk_setting.voxel_size.log2();
//         if log_2_size.fract() != 0.0 {
//             error!("chunk_size must be 2^n");
//             validation = false;
//         }

//         validation
//     }
// }

impl Default for TerrainSetting {
    fn default() -> Self {
        Self {
            qef_solver: true,
            qef_solver_threshold: 0.1,
            qef_stddev: 0.1,
            stitch_seam_scheme: StitchSeamScheme::NeighborConnect,
            chunk_setting: TerrainChunkSetting {
                voxel_size: 1.0,
                voxel_count: 16,
            },
            size_setting: TerrainSizeSetting {
                height_range: -8..=16,
            },
            clipmap_config: ClipmapConfig::default(),
        }
    }
}

impl TerrainSetting {
    pub fn get_chunk_size_by_lod(&self, lod: u8) -> f32 {
        self.chunk_setting.get_chunk_size() * 2u32.pow(lod as u32) as f32
    }

    pub fn get_voxel_size_by_lod(&self, lod: u8) -> f32 {
        self.chunk_setting.voxel_size * 2u32.pow(lod as u32) as f32
    }

    pub fn get_voxel_count_in_chunk(&self) -> u32 {
        self.chunk_setting.voxel_count as u32
    }

    pub fn is_in_height_range(&self, height: i32) -> bool {
        self.size_setting.height_range.contains(&height)
    }

    /*
     * 获取指定 LOD 级别的 clipmap 半径，单位是 chunk 数量
     */
    pub fn get_clipmap_radius_by_lod(&self, lod: u8) -> u8 {
        self.clipmap_config.get_radius_by_lod(lod)
    }
}
