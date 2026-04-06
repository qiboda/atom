use crate::effect::tag::{
    EffectAbortDisableLayerTagContainer, EffectAbortRequiredLayerTagContainer,
    EffectAddedLayerTagContainer, EffectRemovedLayerTagContainer,
    EffectStartDisableLayerTagContainer, EffectStartRequiredLayerTagContainer,
};
use bevy::{prelude::Bundle, reflect::Reflect};

#[derive(Debug, Default, Bundle, Reflect)]
pub struct EffectStartTagBundle {
    pub required_layer_tag: EffectStartRequiredLayerTagContainer,
    pub disable_layer_tag: EffectStartDisableLayerTagContainer,
    pub added_layer_tag: EffectAddedLayerTagContainer,
    pub removed_layer_tag: EffectRemovedLayerTagContainer,
}

#[derive(Debug, Default, Bundle, Reflect)]
pub struct EffectAbortTagBundle {
    pub required_layer_tag: EffectAbortRequiredLayerTagContainer,
    pub disable_layer_tag: EffectAbortDisableLayerTagContainer,
}
