use bevy::prelude::{App, Plugin};

/// layer state manager.
///
/// support runtime add/remove any layer state.
/// support any layer state with data.
pub mod layertag;
pub mod registry;
pub mod tag;

#[derive(Default)]
pub struct LayerTagPlugin;

impl Plugin for LayerTagPlugin {
    fn build(&self, _app: &mut App) {}
}
