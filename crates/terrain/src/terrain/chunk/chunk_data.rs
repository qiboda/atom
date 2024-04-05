use bevy::{prelude::Component, reflect::Reflect};

#[derive(Debug, Default, Component, Reflect)]
pub struct TerrainChunkData {
    pub loaded: bool,
    pub lod: u8,
}

impl TerrainChunkData {}
