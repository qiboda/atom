use bevy::prelude::{App, Plugin};

pub mod category;
pub mod ecology_layer;
pub mod ecology_set;
pub mod layer;

#[derive(Debug)]
struct EcologyPlugin;

impl Plugin for EcologyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ecology_layer::EcologyLayerPlugin);
    }
}

#[derive(Debug)]
pub enum EcologyType {
    Forest,
    Desert,
}
