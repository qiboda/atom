use bevy::{prelude::*, utils::HashMap};
use serde::{Deserialize, Serialize};
use settings::{Setting, SettingValidate};
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::chunk_mgr::chunk::chunk_lod::{LodType, OctreeDepthType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainChunkSetting {
    /// chunk大小
    pub chunk_size: f32,
    /// chunk octree的深度
    pub depth: OctreeDepthType,
    /// 是否启用octree的节点收缩
    pub qef_solver: bool,
    /// octree的深度对应的qef的阈值，小于这个阈值，则可以收缩节点。
    pub qef_solver_threshold: HashMap<OctreeDepthType, f32>,
    /// qef solver的单位标准差
    pub qef_stddev: f32,
}

impl TerrainChunkSetting {
    pub fn get_chunk_size(&self, lod: LodType) -> f32 {
        self.chunk_size * 2.0f32.powi(lod as i32)
    }

    pub fn get_voxel_size(&self, lod: LodType) -> f32 {
        // debug_assert!(lod <= self.depth);
        // 根节点深度为0.
        self.chunk_size / 2.0f32.powi(self.depth as i32 - lod as i32)
    }
}

impl SettingValidate for TerrainChunkSetting {
    fn validate(&self) -> bool {
        let log_2_size = self.chunk_size.log2();
        let mut validation = true;
        if log_2_size.fract() != 0.0 {
            error!("chunk_size must be 2^n");
            validation = false;
        }
        if self.qef_solver_threshold.len() < self.depth as usize {
            error!("qef_solver_threshold.len() < depth");
            validation = false;
        }

        validation
    }
}

impl Default for TerrainChunkSetting {
    fn default() -> Self {
        Self {
            chunk_size: 64.0,
            depth: 6,
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
            qef_stddev: 0.1,
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
pub struct TerrainLodOctreeSetting {
    lod_vec: Vec<TerrainClipMapLod>,
    lod_octree_depth: OctreeDepthType,
}

impl Default for TerrainLodOctreeSetting {
    fn default() -> Self {
        Self {
            lod_vec: vec![
                TerrainClipMapLod::new(2, 0),
                TerrainClipMapLod::new(6, 1),
                TerrainClipMapLod::new(12, 2),
                TerrainClipMapLod::new(24, 3),
                TerrainClipMapLod::new(48, 4),
                TerrainClipMapLod::new(96, 6),
                TerrainClipMapLod::new(192, 7),
                TerrainClipMapLod::new(364, 7),
                TerrainClipMapLod::new(2560000, 7),
            ],
            lod_octree_depth: 8,
        }
    }
}

impl TerrainLodOctreeSetting {
    pub fn get_lod(&self, chebyshev_distance: u64) -> Option<&TerrainClipMapLod> {
        self.lod_vec
            .iter()
            .find(|lod| chebyshev_distance <= lod.chunk_chebyshev_distance)
    }

    pub fn get_depth(&self, chebyshev_distance: u64) -> Option<OctreeDepthType> {
        self.get_lod(chebyshev_distance)
            .map(|x| self.lod_octree_depth - x.lod)
    }

    pub fn get_lod_octree_depth(&self) -> OctreeDepthType {
        self.lod_octree_depth
    }
}

#[derive(Setting, Resource, Debug, Clone, Serialize, Deserialize, TypePath, Asset, Default)]
pub struct TerrainSetting {
    pub chunk_setting: TerrainChunkSetting,
    pub lod_setting: TerrainLodOctreeSetting,
}

impl SettingValidate for TerrainSetting {
    fn validate(&self) -> bool {
        let mut validation = true;
        validation &= self.chunk_setting.validate();
        validation
    }
}

impl TerrainSetting {
    pub fn get_lod_octree_size(&self) -> f32 {
        let lod_octree_depth = self.lod_setting.lod_octree_depth;
        self.chunk_setting.get_chunk_size(lod_octree_depth)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_terrain_chunk_setting() {
        let setting = TerrainChunkSetting {
            chunk_size: 64.0,
            depth: 7,
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
            qef_stddev: 0.1,
        };
        assert_eq!(setting.get_chunk_size(0), 64.0);
        assert_eq!(setting.get_chunk_size(1), 128.0);
        assert_eq!(setting.get_chunk_size(2), 256.0);
        assert_eq!(setting.get_chunk_size(3), 512.0);
        assert_eq!(setting.get_chunk_size(4), 1024.0);
        assert_eq!(setting.get_chunk_size(5), 2048.0);
        assert_eq!(setting.get_chunk_size(6), 4096.0);
        assert_eq!(setting.get_chunk_size(7), 8192.0);
        assert_eq!(setting.get_chunk_size(8), 16384.0);
        assert_eq!(setting.get_chunk_size(9), 32768.0);

        assert_eq!(setting.get_voxel_size(0), 64.0);
        assert_eq!(setting.get_voxel_size(1), 32.0);
        assert_eq!(setting.get_voxel_size(2), 16.0);
        assert_eq!(setting.get_voxel_size(3), 8.0);
        assert_eq!(setting.get_voxel_size(4), 4.0);
        assert_eq!(setting.get_voxel_size(5), 2.0);
        assert_eq!(setting.get_voxel_size(6), 1.0);
        assert_eq!(setting.get_voxel_size(7), 0.5);
        assert_eq!(setting.get_voxel_size(8), 0.25);
        assert_eq!(setting.get_voxel_size(9), 0.125);
    }

    #[test]
    fn test_terrain_lod_octree_setting() {
        let setting = TerrainSetting {
            chunk_setting: TerrainChunkSetting {
                chunk_size: 64.0,
                depth: 7,
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
                qef_stddev: 0.1,
            },
            lod_setting: TerrainLodOctreeSetting {
                lod_vec: vec![
                    TerrainClipMapLod::new(0, 0),
                    TerrainClipMapLod::new(1, 1),
                    TerrainClipMapLod::new(2, 2),
                    TerrainClipMapLod::new(3, 3),
                    TerrainClipMapLod::new(4, 4),
                ],
                lod_octree_depth: 8,
            },
        };
        assert_eq!(setting.get_lod_octree_size(), 16384.0);
    }
}
