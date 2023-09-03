use std::{fmt::Debug, ops::ControlFlow, sync::Arc};

use bevy::prelude::*;

use crate::terrain::chunk::coords::TerrainChunkCoord;

use super::category::EcologyMaterial;

pub mod first;

pub trait Sampler {
    fn sample(&self, chunk_coord: TerrainChunkCoord) -> Option<Arc<dyn EcologyMaterial>>;
}

pub trait EcologyLayer: Sampler + Send + Sync + Debug {}

#[derive(Debug, Component)]
pub struct EcologyLayerSampler {
    pub all_layer: Vec<Box<dyn EcologyLayer>>,
}

impl Sampler for EcologyLayerSampler {
    fn sample(&self, chunk_coord: TerrainChunkCoord) -> Option<Arc<dyn EcologyMaterial>> {
        if let ControlFlow::Break(mat) =
            self.all_layer
                .iter()
                .rev()
                .try_for_each(|layer| match layer.sample(chunk_coord) {
                    mat if mat.is_some() => return ControlFlow::Break(mat),
                    _ => return ControlFlow::Continue(()),
                })
        {
            return mat;
        }
        return None;
    }
}
