use std::{fmt::Debug, ops::ControlFlow, sync::Arc};

use bevy::{math::bounding::Aabb3d, prelude::*};

use terrain_core::chunk::coords::TerrainChunkCoord;

use super::category::EcologyMaterial;

pub mod first;

#[reflect_trait]
pub trait Sampler {
    fn sample(
        &self,
        chunk_coord: TerrainChunkCoord,
        aabb: Aabb3d,
    ) -> Option<Arc<dyn EcologyMaterial>>;
}

#[reflect_trait]
pub trait EcologyLayer: Sampler + Send + Sync + Debug {}

#[derive(Debug, Component)]
pub struct EcologyLayerSampler {
    pub all_layer: Vec<Box<dyn EcologyLayer>>,
}

impl Sampler for EcologyLayerSampler {
    fn sample(
        &self,
        chunk_coord: TerrainChunkCoord,
        aabb: Aabb3d,
    ) -> Option<Arc<dyn EcologyMaterial>> {
        if let ControlFlow::Break(mat) = self.all_layer.iter().rev().try_for_each(|layer| {
            match layer.sample(chunk_coord, aabb) {
                mat if mat.is_some() => ControlFlow::Break(mat),
                _ => ControlFlow::Continue(()),
            }
        }) {
            return mat;
        }
        None
    }
}
