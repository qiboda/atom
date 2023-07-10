use bevy::prelude::Bundle;

use super::terrain::TerrainData;

#[derive(Bundle, Default)]
pub struct TerrainBundle {
    pub terrain_data: TerrainData,
}
