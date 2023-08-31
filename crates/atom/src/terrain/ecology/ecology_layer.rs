use bevy::prelude::{App, Plugin};

#[derive(Debug)]
pub struct EcologyLayer {}

#[derive(Debug)]
pub struct EcologyLayerPlugin;

impl Plugin for EcologyLayerPlugin {
    fn build(&self, app: &mut App) {}
}
