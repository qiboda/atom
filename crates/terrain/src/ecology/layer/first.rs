use std::sync::Arc;

use bevy::math::bounding::Aabb3d;

use crate::ecology::category::EcologyMaterial;

use super::{EcologyLayer, Sampler};

#[derive(Debug)]
pub struct FirstLayer {
    pub forest_material: Arc<dyn EcologyMaterial>,
}

impl Sampler for FirstLayer {
    fn sample(&self, _aabb: Aabb3d) -> Option<Arc<dyn EcologyMaterial>> {
        Some(self.forest_material.clone())
    }
}

impl EcologyLayer for FirstLayer {}
