use std::{fmt::Debug, ops::ControlFlow, sync::Arc};

use bevy::prelude::*;

use crate::terrain::{chunk::coords::TerrainChunkCoord, isosurface::dc::CellExtent};

use super::category::EcologyMaterial;

pub mod first;

pub trait Sampler {
    fn sample(
        &self,
        chunk_coord: TerrainChunkCoord,
        cell_extent: CellExtent,
    ) -> Option<Arc<dyn EcologyMaterial>>;
}

pub trait EcologyLayer: Sampler + Send + Sync + Debug {}

#[derive(Debug, Component)]
pub struct EcologyLayerSampler {
    pub all_layer: Vec<Box<dyn EcologyLayer>>,
}

impl Sampler for EcologyLayerSampler {
    fn sample(
        &self,
        chunk_coord: TerrainChunkCoord,
        cell_extent: CellExtent,
    ) -> Option<Arc<dyn EcologyMaterial>> {
        if let ControlFlow::Break(mat) = self.all_layer.iter().rev().try_for_each(|layer| {
            match layer.sample(chunk_coord, cell_extent) {
                mat if mat.is_some() => ControlFlow::Break(mat),
                _ => ControlFlow::Continue(()),
            }
        }) {
            return mat;
        }
        None
    }
}
