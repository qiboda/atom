use atom_datatables::effect::TbAbilityRow;
use atom_layertag::container_op::{
    LayerTagContainerConditionRequired, LayerTagContainerConditionWithout, LayerTagContainerOpAdd,
    LayerTagContainerOpRemove,
};
use bevy::prelude::*;

use crate::{
    ability::node::ability_entry::EffectNodeAbilityEntry,
    graph::{
        event::{EffectGraphExecEvent, EffectGraphRemoveEvent, EffectGraphTickableEvent},
        state::EffectGraphState,
    },
    stateset::StateLayerTagContainer,
};

use super::{
    comp::{Ability, AbilityExecuteState},
    layertag::tag::{
        AbilityAbortDisableLayerTagContainer, AbilityAbortRequiredLayerTagContainer,
        AbilityAddedLayerTagContainer, AbilityRemovedLayerTagContainer,
        AbilityStartDisableLayerTagContainer, AbilityStartRequiredLayerTagContainer,
    },
};

use crate::graph::event::EffectGraphAddEvent;

#[derive(Debug, Event)]
pub struct AbilityReadyEvent;

#[derive(Debug, EntityEvent)]
pub struct AbilityStartEvent {
    #[event_target]
    pub ability_entity: Entity,
}

// 需要后续处理，等待技能执行完毕。
// TODO: 如果处于激活状态下，需要触发中断事件。
#[derive(Debug, EntityEvent)]
pub struct AbilityRemoveEvent {
    #[event_target]
    pub ability_entity: Entity,
}

// 强制中断技能。
#[derive(Debug, EntityEvent)]
pub struct AbilityAbortEvent {
    #[event_target]
    pub ability_entity: Entity,
}

#[derive(Debug, EntityEvent)]
pub struct AbilityTickableEvent {
    pub tickable: bool,
    #[event_target]
    pub ability_entity: Entity,
}

#[allow(clippy::type_complexity)]
pub fn trigger_ability_ready(
    trigger: On<AbilityReadyEvent>,
    state_set_query: Query<&StateLayerTagContainer>,
    mut commands: Commands,
    ability_query: Query<
        (
            &ChildOf,
            &AbilityExecuteState,
            &AbilityStartRequiredLayerTagContainer,
            &AbilityStartDisableLayerTagContainer,
        ),
        With<Ability>,
    >,
) {
    let ability_entity = trigger.observer();

    if let Ok((parent, state, required_tag, disable_tag)) = ability_query.get(ability_entity) {
        if *state == AbilityExecuteState::ToRemove {
            return;
        }

        match state_set_query.get(parent.parent()) {
            Ok(state_layer_tag_container) => {
                let can_start = state_layer_tag_container
                    .0
                    .condition(LayerTagContainerConditionRequired, &required_tag.0)
                    && state_layer_tag_container
                        .0
                        .condition(LayerTagContainerConditionWithout, &disable_tag.0);

                if can_start {
                    info!("trigger_ability_ready: {:?}", ability_entity);
                    commands.trigger(EffectGraphExecEvent {
                        entry_exec_pin: EffectNodeAbilityEntry::OUTPUT_EXEC_READY.into(),
                        execute_in_graph_state: Some(EffectGraphState::Inactive),
                        slot_value_map: None,
                        ability_entity,
                    });
                }
            }
            Err(e) => {
                warn!(
                    "trigger_ability_ready: state layer tag container not found for entity {:?}: {:?}",
                    parent.parent(),
                    e
                );
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn trigger_ability_start(
    trigger: On<AbilityStartEvent>,
    mut state_set_query: Query<&mut StateLayerTagContainer>,
    mut commands: Commands,
    ability_query: Query<
        (
            &ChildOf,
            &AbilityExecuteState,
            &AbilityAddedLayerTagContainer,
            &AbilityRemovedLayerTagContainer,
        ),
        With<Ability>,
    >,
) {
    let ability_entity = trigger.observer();

    if let Ok((parent, state, added_tag, removed_tag)) = ability_query.get(ability_entity) {
        if *state == AbilityExecuteState::ToRemove {
            return;
        }

        match state_set_query.get_mut(parent.parent()) {
            Ok(mut state_layer_tag_container) => {
                state_layer_tag_container
                    .0
                    .receive_op(LayerTagContainerOpAdd, &added_tag.layer_tag_container);

                state_layer_tag_container
                    .0
                    .receive_op(LayerTagContainerOpRemove, &removed_tag.layer_tag_container);
                info!("trigger_ability_start: {:?}", ability_entity);
                commands.trigger(EffectGraphExecEvent {
                    entry_exec_pin: EffectNodeAbilityEntry::OUTPUT_EXEC_START.into(),
                    execute_in_graph_state: Some(EffectGraphState::Inactive),
                    slot_value_map: None,
                    ability_entity,
                });
            }
            Err(e) => {
                warn!(
                    "trigger_ability_start: state layer tag container not found for entity {:?}: {:?}",
                    parent.parent(),
                    e
                );
            }
        }
    }
}

pub fn trigger_ability_remove(
    trigger: On<AbilityRemoveEvent>,
    mut commands: Commands,
    mut ability_query: Query<&mut AbilityExecuteState, With<Ability>>,
) {
    let ability_entity = trigger.observer();
    if let Ok(mut state) = ability_query.get_mut(ability_entity) {
        info!("trigger_ability_remove: {:?}", ability_entity);
        *state = AbilityExecuteState::ToRemove;
        commands.trigger(EffectGraphRemoveEvent { ability_entity });
    }
}

// TODO: 如果技能删除或者中断或者结束，需要将技能添加的状态层标签移除。
// pub fn Ability_tag_revert_apply_system(
//     mut state_set_query: Query<&mut StateLayerTagContainer>,
//     query: Query<(
//         &ChildOf,
//         &AbilityState,
//         &AbilityAddedLayerTagContainer,
//         &AbilityRemovedLayerTagContainer,
//     )>,
// ) {
//     for (parent, effect_state, added_tag, removed_tag) in query.iter() {
//         if *effect_state == AbilityState::BeforeInactive {
//             let mut state_layer_tag_container = state_set_query.get_mut(parent.get()).unwrap();

//             if added_tag.revert == AbilityLayerTagContainerRevert::Yes {
//                 state_layer_tag_container
//                     .0
//                     .receive_op(LayerTagContainerOpRemove, &added_tag.layer_tag_container);
//             }

//             if removed_tag.revert == AbilityLayerTagContainerRevert::Yes {
//                 state_layer_tag_container
//                     .0
//                     .receive_op(LayerTagContainerOpAdd, &removed_tag.layer_tag_container);
//             }
//         }
//     }
// }

pub fn trigger_ability_abort(
    trigger: On<AbilityAbortEvent>,
    mut commands: Commands,
    state_set_query: Query<&StateLayerTagContainer>,
    mut ability_query: Query<
        (
            &ChildOf,
            &AbilityExecuteState,
            &AbilityAbortRequiredLayerTagContainer,
            &AbilityAbortDisableLayerTagContainer,
        ),
        With<Ability>,
    >,
) {
    let ability_entity = trigger.observer();
    if let Ok((parent, state, required_tag, disable_tag)) = ability_query.get_mut(ability_entity) {
        if *state == AbilityExecuteState::ToRemove {
            return;
        }

        match state_set_query.get(parent.parent()) {
            Ok(state_layer_tag_container) => {
                let can_abort = state_layer_tag_container
                    .0
                    .condition(LayerTagContainerConditionRequired, &required_tag.0)
                    && state_layer_tag_container
                        .0
                        .condition(LayerTagContainerConditionWithout, &disable_tag.0);

                if can_abort {
                    info!("trigger_ability_abort: {:?}", ability_entity);
                    commands.trigger(EffectGraphExecEvent {
                        entry_exec_pin: EffectNodeAbilityEntry::OUTPUT_EXEC_ABORT.into(),
                        execute_in_graph_state: Some(EffectGraphState::Inactive),
                        slot_value_map: None,
                        ability_entity,
                    });
                }
            }
            Err(e) => {
                warn!(
                    "trigger_ability_abort: state layer tag container not found for entity {:?}: {:?}",
                    parent.parent(),
                    e
                );
            }
        }
    }
}

pub fn trigger_ability_tickable(
    trigger: On<AbilityTickableEvent>,
    mut commands: Commands,
    mut ability_query: Query<&AbilityExecuteState, With<Ability>>,
) {
    let ability_entity = trigger.observer();
    if let Ok(state) = ability_query.get_mut(ability_entity) {
        if *state == AbilityExecuteState::ToRemove {
            return;
        }

        info!("trigger_ability_abort: {:?}", ability_entity);
        commands.trigger(EffectGraphTickableEvent {
            tickable: trigger.event().tickable,
            ability_entity,
        });
    }
}

// add to ability entity observer
pub fn trigger_ability_add(
    trigger: On<Add, Ability>,
    mut commands: Commands,
    query: Query<&TbAbilityRow, With<Ability>>,
) {
    let ability_entity = trigger.observer();
    match query.get(ability_entity) {
        Ok(ability_row) => {
            if let Some(data) = ability_row.data.clone() {
                commands.trigger(EffectGraphAddEvent {
                    graph_class: data.graph_class.clone(),
                    ability_entity,
                });
            }
        }
        Err(e) => {
            warn!(
                "trigger_ability_add: ability row not found for entity {:?}: {:?}",
                ability_entity, e
            );
        }
    }
}
