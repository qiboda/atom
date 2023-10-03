use bevy::{prelude::Component, reflect::Reflect};
use layertag::container::LayerTagContainer;

#[derive(Component, Default, Debug, Reflect)]
pub struct StateLayerTagContainer(pub LayerTagContainer);
