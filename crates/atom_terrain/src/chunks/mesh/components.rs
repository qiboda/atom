use bevy::{prelude::*, render::extract_component::ExtractComponent};

#[derive(Component, ExtractComponent, Default, Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainChunkMeshingState {
    #[default]
    Idle,
    Meshing,
    // TODO: 是否移除？
    Finish,
}
