use bevy::{prelude::*, utils::HashMap};
use datatables::{
    effect::{TbBuff, TbBuffKey, TbBuffRow},
    tables_system_param::TableReader,
};
use layertag::container_op::{
    LayerTagContainerConditionRequired, LayerTagContainerConditionWithout, LayerTagContainerOpAdd,
    LayerTagContainerOpRemove,
};

use crate::{
    buff::{bundle::BuffBundle, node::buff_entry::EffectNodeBuffEntry},
    graph::{
        blackboard::EffectValue,
        event::{
            EffectGraphAddEvent, EffectGraphExecEvent, EffectGraphRemoveEvent,
            EffectGraphTickableEvent,
        },
        node::pin::EffectNodeSlot,
        state::EffectGraphState,
    },
    stateset::{StateLayerTagContainer, StateLayerTagRegistry},
};

use super::{
    layer::BuffLayer,
    layertag::tag::{
        BuffAbortDisableLayerTagContainer, BuffAbortRequiredLayerTagContainer,
        BuffAddedLayerTagContainer, BuffRemovedLayerTagContainer,
        BuffStartDisableLayerTagContainer, BuffStartRequiredLayerTagContainer,
    },
    state::{Buff, BuffExecuteState},
};

#[derive(Debug, Event)]
pub struct BuffAddEvent {
    pub owner_entity: Entity,
    pub buff_id: TbBuffKey,
}

#[derive(Debug, Event)]
pub struct BuffReadyEvent;

#[derive(Debug, Event)]
pub struct BuffStartEvent;

#[derive(Debug, Event)]
pub struct BuffAbortEvent;

/// TODO: 如果激活，则abort。
#[derive(Debug, Event)]
pub struct BuffRemoveEvent;

#[derive(Debug, Event)]
pub struct BuffTickableEvent {
    pub tickable: bool,
}

pub fn trigger_buff_on_add(
    trigger: Trigger<OnAdd, Buff>,
    mut commands: Commands,
    query: Query<&TbBuffRow, With<Buff>>,
) {
    let buff_entity = trigger.entity();
    let buff_row = query.get(buff_entity).unwrap();

    if let Some(data) = buff_row.data.clone() {
        commands.trigger_targets(
            EffectGraphAddEvent {
                graph_class: data.graph_class.clone(),
            },
            buff_entity,
        );
    }
}

#[allow(clippy::type_complexity)]
pub fn trigger_buff_ready(
    triger: Trigger<BuffReadyEvent>,
    state_set_query: Query<&StateLayerTagContainer>,
    mut commands: Commands,
    buff_query: Query<
        (
            &Parent,
            &BuffExecuteState,
            &BuffStartRequiredLayerTagContainer,
            &BuffStartDisableLayerTagContainer,
        ),
        With<Buff>,
    >,
) {
    let buff_entity = triger.entity();

    if let Ok((parent, state, required_tag, disable_tag)) = buff_query.get(buff_entity) {
        if *state == BuffExecuteState::ToRemove {
            return;
        }

        let state_layer_tag_container = state_set_query.get(parent.get()).unwrap();

        let can_start = state_layer_tag_container
            .0
            .condition(LayerTagContainerConditionRequired, &required_tag.0)
            && state_layer_tag_container
                .0
                .condition(LayerTagContainerConditionWithout, &disable_tag.0);

        if can_start {
            info!("trigger_buff_ready: {:?}", buff_entity);
            commands.trigger_targets(
                EffectGraphExecEvent {
                    entry_exec_pin: EffectNodeBuffEntry::OUTPUT_EXEC_READY.into(),
                    execute_in_graph_state: Some(EffectGraphState::Inactive),
                    slot_value_map: None,
                },
                buff_entity,
            );
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn trigger_buff_start(
    triger: Trigger<BuffStartEvent>,
    mut state_set_query: Query<&mut StateLayerTagContainer>,
    mut commands: Commands,
    buff_query: Query<
        (
            &Parent,
            &BuffExecuteState,
            &BuffAddedLayerTagContainer,
            &BuffRemovedLayerTagContainer,
        ),
        With<Buff>,
    >,
) {
    let buff_entity = triger.entity();

    if let Ok((parent, state, added_tag, removed_tag)) = buff_query.get(buff_entity) {
        if *state == BuffExecuteState::ToRemove {
            return;
        }

        let mut state_layer_tag_container = state_set_query.get_mut(parent.get()).unwrap();

        state_layer_tag_container
            .0
            .receive_op(LayerTagContainerOpAdd, &added_tag.layer_tag_container);

        state_layer_tag_container
            .0
            .receive_op(LayerTagContainerOpRemove, &removed_tag.layer_tag_container);
        info!("trigger_buff_start: {:?}", buff_entity);
        commands.trigger_targets(
            EffectGraphExecEvent {
                entry_exec_pin: EffectNodeBuffEntry::OUTPUT_EXEC_START.into(),
                execute_in_graph_state: Some(EffectGraphState::Inactive),
                slot_value_map: None,
            },
            buff_entity,
        );
    }
}

pub fn trigger_buff_remove(
    triger: Trigger<BuffRemoveEvent>,
    mut commands: Commands,
    mut buff_query: Query<&mut BuffExecuteState, With<Buff>>,
) {
    let buff_entity = triger.entity();
    if let Ok(mut state) = buff_query.get_mut(buff_entity) {
        info!("trigger_buff_remove: {:?}", buff_entity);
        *state = BuffExecuteState::ToRemove;
        commands.trigger_targets(EffectGraphRemoveEvent, buff_entity);
    }
}

pub fn trigger_buff_abort(
    triger: Trigger<BuffAbortEvent>,
    mut commands: Commands,
    state_set_query: Query<&StateLayerTagContainer>,
    mut buff_query: Query<
        (
            &Parent,
            &BuffExecuteState,
            &BuffAbortRequiredLayerTagContainer,
            &BuffAbortDisableLayerTagContainer,
        ),
        With<Buff>,
    >,
) {
    let buff_entity = triger.entity();
    if let Ok((parent, state, required_tag, disable_tag)) = buff_query.get_mut(buff_entity) {
        if *state == BuffExecuteState::ToRemove {
            return;
        }

        let state_layer_tag_container = state_set_query.get(parent.get()).unwrap();
        let can_abort = state_layer_tag_container
            .0
            .condition(LayerTagContainerConditionRequired, &required_tag.0)
            && state_layer_tag_container
                .0
                .condition(LayerTagContainerConditionWithout, &disable_tag.0);

        if can_abort {
            info!("trigger_buff_abort: {:?}", buff_entity);
            commands.trigger_targets(
                EffectGraphExecEvent {
                    entry_exec_pin: EffectNodeBuffEntry::OUTPUT_EXEC_ABORT.into(),
                    execute_in_graph_state: Some(EffectGraphState::Inactive),
                    slot_value_map: None,
                },
                buff_entity,
            );
        }
    }
}

pub fn trigger_buff_tickable(
    triger: Trigger<BuffTickableEvent>,
    mut commands: Commands,
    mut buff_query: Query<&BuffExecuteState, With<Buff>>,
) {
    let buff_entity = triger.entity();
    if let Ok(state) = buff_query.get_mut(buff_entity) {
        if *state == BuffExecuteState::ToRemove {
            return;
        }

        info!("trigger_buff_abort: {:?}", buff_entity);
        commands.trigger_targets(
            EffectGraphTickableEvent {
                tickable: triger.event().tickable,
            },
            buff_entity,
        );
    }
}

pub fn trigger_buff_add_event(
    trigger: Trigger<BuffAddEvent>,
    mut commands: Commands,
    table_reader: TableReader<TbBuff>,
    owner_query: Query<&Children>,
    mut query: Query<(&mut BuffLayer, &TbBuffRow), With<Buff>>,
    state_registry: Res<StateLayerTagRegistry>,
) {
    let event = trigger.event();
    info!("trigger_buff_add: {:?}", event.buff_id);

    let Some(new_buff_data) = table_reader.get_row(&event.buff_id) else {
        return;
    };

    if let Ok(children) = owner_query.get(event.owner_entity) {
        for child in children {
            if let Ok((mut buff_layer, buff_row)) = query.get_mut(*child) {
                if buff_row.key() == &event.buff_id {
                    buff_layer.add_layer(1);

                    let mut slot_value_map = HashMap::new();
                    slot_value_map.insert(
                        EffectNodeSlot::new::<i32>(EffectNodeBuffEntry::OUTPUT_SLOT_ADDED_LAYER),
                        EffectValue::I32(1),
                    );
                    commands.trigger_targets(
                        EffectGraphExecEvent {
                            entry_exec_pin: EffectNodeBuffEntry::OUTPUT_EXEC_ADD_LAYER.into(),
                            execute_in_graph_state: Some(EffectGraphState::Active),
                            slot_value_map: Some(slot_value_map),
                        },
                        *child,
                    );
                    return;
                }
            }
        }

        let buff_bundle = BuffBundle::new(
            TbBuffRow {
                key: event.buff_id,
                data: Some(new_buff_data),
            },
            &state_registry,
        );

        commands.spawn(buff_bundle).set_parent(event.owner_entity);
    }
}
