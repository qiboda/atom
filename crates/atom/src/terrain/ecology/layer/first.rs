use std::sync::Arc;

use crate::terrain::{
    chunk::coords::TerrainChunkCoord, ecology::category::EcologyMaterial,
    isosurface::dc::CellExtent,
};

use super::{EcologyLayer, Sampler};

#[derive(Debug)]
pub struct FirstLayer {
    pub forest_material: Arc<dyn EcologyMaterial>,
}

impl Sampler for FirstLayer {
    fn sample(
        &self,
        _chunk_coord: TerrainChunkCoord,
        _cell_extent: CellExtent,
    ) -> Option<Arc<dyn EcologyMaterial>> {
        Some(self.forest_material.clone())
    }
}

impl EcologyLayer for FirstLayer {}
