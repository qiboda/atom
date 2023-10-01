use bevy::{prelude::Bundle, reflect::Reflect};
use layertag::container::{RequiredLayerTagContainer, DisableLayerTagContainer};

use super::AbilityEffect;

#[derive(Debug, Default, Bundle, Reflect)]
pub struct AbilityEffectBundle {
    pub effect: AbilityEffect,
    pub required_layer_tag: RequiredLayerTagContainer,
    pub disable_layer_tag: DisableLayerTagContainer,
}
