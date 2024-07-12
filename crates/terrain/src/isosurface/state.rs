use bevy::prelude::*;

#[derive(PartialEq, Eq, Debug, Clone, Hash, Component)]
pub enum IsosurfaceState {
    GenMeshInfo,
    CreateMesh,
    UpdateLod,
    Done,
}
