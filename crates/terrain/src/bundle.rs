use bevy::{prelude::*, transform::bundles::TransformBundle};

use super::chunk::chunk_mapper::TerrainChunkMapper;

// root entity
#[derive(Debug, Component, Default, Reflect)]
pub struct Terrain;

#[derive(Bundle, Default)]
pub struct TerrainBundle {
    pub chunk_mapper: TerrainChunkMapper,
    pub transform_bundle: TransformBundle,
    pub visibility_bundle: VisibilityBundle,
    pub terrain: Terrain,
}
