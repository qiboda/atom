use std::sync::Arc;

use crate::ecology::category::EcologyMaterial;
use bevy::math::bounding::Aabb3d;
use terrain_core::chunk::coords::TerrainChunkCoord;

use super::{EcologyLayer, Sampler};

#[derive(Debug)]
pub struct FirstLayer {
    pub forest_material: Arc<dyn EcologyMaterial>,
}

impl Sampler for FirstLayer {
    fn sample(
        &self,
        _chunk_coord: TerrainChunkCoord,
        _aabb: Aabb3d,
    ) -> Option<Arc<dyn EcologyMaterial>> {
        Some(self.forest_material.clone())
    }
}

impl EcologyLayer for FirstLayer {}
