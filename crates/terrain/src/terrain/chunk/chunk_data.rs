use bevy::prelude::Component;

#[derive(Debug, Default, Component)]
pub struct TerrainChunkData {
    pub loaded: bool,
    pub lod: u8,
}

impl TerrainChunkData {}
