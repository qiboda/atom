use std::sync::Arc;

use crate::terrain::{chunk::coords::TerrainChunkCoord, ecology::category::EcologyMaterial};

use super::{EcologyLayer, Sampler};

#[derive(Debug)]
pub struct FirstLayer {
    pub forest_material: Arc<dyn EcologyMaterial>,
}

impl Sampler for FirstLayer {
    fn sample(&self, chunk_coord: TerrainChunkCoord) -> Option<Arc<dyn EcologyMaterial>> {
        return Some(self.forest_material.clone());
    }
}

impl EcologyLayer for FirstLayer {}
