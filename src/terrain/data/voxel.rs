use bevy::prelude::Component;

#[derive(Debug, PartialEq, Default, Eq, Clone, Copy)]
pub enum VoxelLoadState {
    #[default]
    Unloaded,
    ToLoad,
    Loaded,
    ToUnload,
}

#[derive(Debug, Component, Default)]
pub struct TerrainVoxelData {
    pub load_state: VoxelLoadState,
}

impl TerrainVoxelData {
    pub fn set_load_state(&mut self, load_state: VoxelLoadState) {
        if self.load_state == load_state {
            return;
        }

        match (self.load_state, load_state) {
            (VoxelLoadState::ToLoad, VoxelLoadState::Unloaded) => todo!(),
            (VoxelLoadState::ToLoad, VoxelLoadState::ToLoad) => todo!(),
            (VoxelLoadState::ToLoad, VoxelLoadState::Loaded) => todo!(),
            (VoxelLoadState::ToLoad, VoxelLoadState::ToUnload) => todo!(),

            (VoxelLoadState::Loaded, VoxelLoadState::Unloaded) => todo!(),
            (VoxelLoadState::Loaded, VoxelLoadState::ToLoad) => todo!(),
            (VoxelLoadState::Loaded, VoxelLoadState::Loaded) => todo!(),
            (VoxelLoadState::Loaded, VoxelLoadState::ToUnload) => todo!(),

            (VoxelLoadState::ToUnload, VoxelLoadState::Unloaded) => todo!(),
            (VoxelLoadState::ToUnload, VoxelLoadState::ToLoad) => todo!(),
            (VoxelLoadState::ToUnload, VoxelLoadState::Loaded) => todo!(),
            (VoxelLoadState::ToUnload, VoxelLoadState::ToUnload) => todo!(),
            _ => {}
        }

        self.load_state = load_state;
    }
}
