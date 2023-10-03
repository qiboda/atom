use std::ops::Not;

use bevy::prelude::*;
use layertag::container::{
    LayerTagContainer, LayerTagContainerConditionRequired, LayerTagContainerConditionWithout,
    LayerTagContainerOpAdd, LayerTagContainerOpRemove,
};

use crate::stateset::StateLayerTagContainer;

use super::state::EffectState;

#[derive(Component, Debug, Default, Reflect)]
pub struct EffectStartRequiredLayerTagContainer(pub LayerTagContainer);

#[derive(Component, Debug, Default, Reflect)]
pub struct EffectStartDisableLayerTagContainer(pub LayerTagContainer);

#[derive(Component, Debug, Default, Reflect)]
pub struct EffectAbortRequiredLayerTagContainer(pub LayerTagContainer);

#[derive(Component, Debug, Default, Reflect)]
pub struct EffectAbortDisableLayerTagContainer(pub LayerTagContainer);

#[derive(Debug, Default, Reflect, PartialEq, Eq)]
pub enum EffectLayerTagContainerRevert {
    #[default]
    No,
    Yes,
}

#[derive(Component, Debug, Default, Reflect)]
pub struct EffectAddedLayerTagContainer {
    pub layer_tag_container: LayerTagContainer,
    pub revert: EffectLayerTagContainerRevert,
}

#[derive(Component, Debug, Default, Reflect)]
pub struct EffectRemovedLayerTagContainer {
    pub layer_tag_container: LayerTagContainer,
    pub revert: EffectLayerTagContainerRevert,
}

pub fn effect_tag_start_check_system(
    state_set_query: Query<&StateLayerTagContainer>,
    mut query: Query<(
        &Parent,
        &mut EffectState,
        &EffectStartRequiredLayerTagContainer,
        &EffectStartDisableLayerTagContainer,
    )>,
) {
    for (parent, mut effect_state, required_tag, disable_tag) in query.iter_mut() {
        if *effect_state == EffectState::CheckCanActive {
            let state_layer_tag_container = state_set_query.get(parent.get()).unwrap();

            let can_start = state_layer_tag_container
                .0
                .condition(LayerTagContainerConditionRequired, &required_tag.0)
                && state_layer_tag_container
                    .0
                    .condition(LayerTagContainerConditionWithout, &disable_tag.0);
            if can_start.not() {
                *effect_state = EffectState::Unactived;
            }
        }
    }
}

pub fn effect_tag_start_apply_system(
    mut state_set_query: Query<&mut StateLayerTagContainer>,
    query: Query<(
        &Parent,
        &EffectState,
        &EffectAddedLayerTagContainer,
        &EffectRemovedLayerTagContainer,
    )>,
) {
    for (parent, effect_state, added_tag, removed_tag) in query.iter() {
        if *effect_state == EffectState::ActiveBefore {
            let mut state_layer_tag_container = state_set_query.get_mut(parent.get()).unwrap();

            state_layer_tag_container
                .0
                .receive_op(LayerTagContainerOpAdd, &added_tag.layer_tag_container);

            state_layer_tag_container
                .0
                .receive_op(LayerTagContainerOpRemove, &removed_tag.layer_tag_container);
        }
    }
}

pub fn effect_tag_revert_apply_system(
    mut state_set_query: Query<&mut StateLayerTagContainer>,
    query: Query<(
        &Parent,
        &EffectState,
        &EffectAddedLayerTagContainer,
        &EffectRemovedLayerTagContainer,
    )>,
) {
    for (parent, effect_state, added_tag, removed_tag) in query.iter() {
        if *effect_state == EffectState::BeforeUnactived {
            let mut state_layer_tag_container = state_set_query.get_mut(parent.get()).unwrap();

            if added_tag.revert == EffectLayerTagContainerRevert::Yes {
                state_layer_tag_container
                    .0
                    .receive_op(LayerTagContainerOpRemove, &added_tag.layer_tag_container);
            }

            if removed_tag.revert == EffectLayerTagContainerRevert::Yes {
                state_layer_tag_container
                    .0
                    .receive_op(LayerTagContainerOpAdd, &removed_tag.layer_tag_container);
            }
        }
    }
}
