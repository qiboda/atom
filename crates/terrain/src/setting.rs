use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};
use settings::{Setting, SettingValidate};
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::chunk_mgr::chunk::chunk_lod::LodType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainChunkSetting {
    /// chunk大小
    pub chunk_size: f32,
    /// 体素大小
    pub voxel_size: f32,
    /// 是否启用octre的节点收缩
    pub qef_solver: bool,
    /// octree的深度对应的qef的阈值，小于这个阈值，则可以收缩节点。
    pub qef_solver_threshold: HashMap<u16, f32>,
    /// qef solver的位置标准差
    pub qef_pos_stddev: f32,
    /// qef solver的法向量标准差
    pub qef_normal_stddev: f32,
}

impl SettingValidate for TerrainChunkSetting {
    fn validate(&self) -> bool {
        let depth = (self.chunk_size / self.voxel_size).log2();
        let mut validation = true;
        if depth.fract() != 0.0 {
            error!("chunk_size / voxel_size must be 2^n");
            validation = false;
        }
        if self.qef_solver_threshold.len() < depth as usize {
            error!("qef_solver_threshold.len() < depth");
            validation = false;
        }

        validation
    }
}

impl Default for TerrainChunkSetting {
    fn default() -> Self {
        let voxel_size = 0.25;
        Self {
            chunk_size: 16.0,
            voxel_size,
            qef_solver: true,
            qef_solver_threshold: HashMap::from([
                (0, 0.05),
                (1, 0.1),
                (2, 0.5),
                (3, 1.0),
                (4, 2.0),
                (5, 4.0),
                (6, 8.0),
                (7, 10.0),
                (8, 10.0),
                (9, 10.0),
            ]),
            qef_pos_stddev: 0.1 * voxel_size,
            qef_normal_stddev: 0.1,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TerrainClipMapLod {
    /// relative to the active camera chunk coord
    pub chunk_chebyshev_distance: u64,
    /// lod is octree depth
    pub lod: LodType,
}

impl TerrainClipMapLod {
    pub fn new(chunk_chebyshev_distance: u64, lod: LodType) -> Self {
        Self {
            chunk_chebyshev_distance,
            lod,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainClipMapSetting {
    lods: Vec<TerrainClipMapLod>,
}

impl Default for TerrainClipMapSetting {
    fn default() -> Self {
        Self {
            lods: vec![
                TerrainClipMapLod::new(0, 0),
                TerrainClipMapLod::new(1, 1),
                TerrainClipMapLod::new(2, 2),
                TerrainClipMapLod::new(3, 3),
                TerrainClipMapLod::new(4, 4),
            ],
        }
    }
}

impl TerrainClipMapSetting {
    pub fn get_lod(
        &self,
        terrain_chunk_coord_offset: TerrainChunkCoord,
    ) -> Option<&TerrainClipMapLod> {
        let chunk_coord_offset = terrain_chunk_coord_offset.chebyshev_distance();
        self.lods
            .iter()
            .find(|lod| chunk_coord_offset <= lod.chunk_chebyshev_distance)
    }
}

#[derive(Setting, Resource, Debug, Clone, Serialize, Deserialize, TypePath, Asset, Default)]
pub struct TerrainSetting {
    pub chunk_setting: TerrainChunkSetting,
    pub clipmap_setting: TerrainClipMapSetting,
}

impl SettingValidate for TerrainSetting {
    fn validate(&self) -> bool {
        let mut validation = true;
        validation &= self.chunk_setting.validate();
        validation
    }
}
