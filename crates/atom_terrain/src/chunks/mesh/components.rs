use bevy::{prelude::*, render::extract_component::ExtractComponent};

#[derive(
    Component, Reflect, ExtractComponent, Default, Hash, Debug, Clone, Copy, PartialEq, Eq,
)]
#[reflect(Component)]
pub enum TerrainChunkMeshingState {
    #[default]
    Idle,
    Meshing,
}

