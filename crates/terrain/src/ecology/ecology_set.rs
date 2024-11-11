use std::sync::Arc;

use bevy::prelude::*;

use super::category::EcologyMaterial;

#[derive(Debug, Resource)]
pub struct EcologyMaterials {
    pub forest_material: Arc<dyn EcologyMaterial>,
    pub desert_material: Arc<dyn EcologyMaterial>,
}
