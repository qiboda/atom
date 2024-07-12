use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use settings::Setting;
use terrain_core::chunk::coords::TerrainChunkCoord;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TerrainChunkSettings {
    pub chunk_size: f32,
    pub voxel_size: f32,
}

impl Default for TerrainChunkSettings {
    fn default() -> Self {
        Self {
            chunk_size: 32.0,
            voxel_size: 1.0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TerrainClipMapLod {
    /// relative to the active camera chunk coord
    pub chunk_chebyshev_distance: u64,
    /// lod is octree depth
    pub lod: u8,
}

impl TerrainClipMapLod {
    pub fn new(chunk_chebyshev_distance: u64, lod: u8) -> Self {
        Self {
            chunk_chebyshev_distance,
            lod,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainClipMapSettings {
    lods: Vec<TerrainClipMapLod>,
}

impl Default for TerrainClipMapSettings {
    fn default() -> Self {
        Self {
            lods: vec![
                TerrainClipMapLod::new(0, 0),
                TerrainClipMapLod::new(1, 1),
                TerrainClipMapLod::new(2, 2),
                TerrainClipMapLod::new(4, 3),
                TerrainClipMapLod::new(8, 4),
            ],
        }
    }
}

impl TerrainClipMapSettings {
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
    pub chunk_settings: TerrainChunkSettings,
    pub clipmap_settings: TerrainClipMapSettings,
}
