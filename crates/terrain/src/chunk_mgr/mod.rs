use bevy::prelude::SystemSet;

pub mod chunk;
pub mod chunk_loader;
pub mod chunk_mapper;
pub mod event;
pub mod plugin;

#[derive(SystemSet, Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TerrainChunkSystemSet {
    UpdateLoader,
    UpdateChunk,
}
