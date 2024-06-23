use bevy::{
    prelude::{Bundle, VisibilityBundle},
    transform::bundles::TransformBundle,
};

use super::terrain_data::TerrainData;

#[derive(Bundle, Default)]
pub struct TerrainBundle {
    pub terrain_data: TerrainData,
    pub transform_bundle: TransformBundle,
    pub visibility_bundle: VisibilityBundle,
}
