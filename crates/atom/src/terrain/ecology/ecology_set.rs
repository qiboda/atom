use std::sync::Arc;

use bevy::prelude::*;

use super::category::EcologyMaterial;

#[derive(Debug, Resource)]
struct EcologyMaterials {
    forest_material: Arc<dyn EcologyMaterial>,
    desert_material: Arc<dyn EcologyMaterial>,
}
