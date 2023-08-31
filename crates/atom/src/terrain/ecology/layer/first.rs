use std::sync::Arc;

use crate::terrain::ecology::category::EcologyMaterial;

use super::EcologyLayer;

#[derive(Debug)]
struct FirstLayer {
    forest_material: Arc<dyn EcologyMaterial>,
}

impl EcologyLayer for FirstLayer {}
