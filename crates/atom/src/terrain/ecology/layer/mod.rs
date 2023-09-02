use std::{ops::ControlFlow, sync::Arc};

use crate::terrain::chunk::coords::TerrainChunkCoord;

use super::category::EcologyMaterial;

pub mod first;

pub trait Sampler {
    fn sample(&self, chunk_coord: TerrainChunkCoord) -> Option<Arc<dyn EcologyMaterial>>;
}

pub trait EcologyLayer: Sampler {}

#[derive(Debug, Component)]
pub struct EcologyLayerSampler {
    all_layer: Vec<Box<dyn EcologyLayer>>,
}

impl Sampler for EcologyLayerSampler {
    fn sample(&self, chunk_coord: TerrainChunkCoord) -> Option<Arc<dyn EcologyMaterial>> {
        return self.all_layer.iter().rev().try_for_each(|layer| {
            if let mat = layer.sample(chunk_coord) {
                return ControlFlow::Break(mat);
            } else {
                return ControlFlow::Continue(None);
            }
        });
    }
}
