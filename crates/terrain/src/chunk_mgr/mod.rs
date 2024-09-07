use bevy::prelude::SystemSet;

pub mod chunk;
pub mod chunk_event;
pub mod chunk_loader;
pub mod chunk_mapper;
pub mod chunk_mesh;
pub mod plugin;

#[derive(SystemSet, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TerrainChunkSystemSet {
    UpdateLoader,
}
