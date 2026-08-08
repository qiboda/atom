//! Buff 事件与 observer：buff 的添加、ready/start/remove/abort/tickable 生命周期。

use atom_layertag::container_op::{
    LayerTagContainerConditionRequired, LayerTagContainerConditionWithout, LayerTagContainerOpAdd,
    LayerTagContainerOpRemove,
};
use bevy::prelude::*;

use crate::{
    buff::node::buff_entry::EffectNodeBuffEntry,
    graph::{
        event::{
            EffectGraphAddEvent, EffectGraphExecEvent, EffectGraphRemoveEvent,
            EffectGraphTickableEvent,
        },
        state::EffectGraphState,
    },
    stateset::StateLayerTagContainer,
};

use super::{
    bundle::BuffConfigData,
    layertag::tag::{
        BuffAbortDisableLayerTagContainer, BuffAbortRequiredLayerTagContainer,
        BuffAddedLayerTagContainer, BuffRemovedLayerTagContainer,
        BuffStartDisableLayerTagContainer, BuffStartRequiredLayerTagContainer,
    },
    state::{Buff, BuffExecuteState},
};

/// 给指定所有者添加 buff 的事件。
#[derive(Debug, Event)]
pub struct BuffAddEvent {
    /// 所有者实体（buff 挂到其下）。
    pub owner_entity: Entity,
    /// buff 数据表主键（`BuffConfig.id`）。
    pub buff_id: i32,
}

/// Buff 就绪事件。
#[derive(Debug, Event)]
pub struct BuffReadyEvent;

/// Buff 开始事件。
#[derive(Debug, Event)]
pub struct BuffStartEvent;

/// Buff 中断事件。
#[derive(Debug, Event)]
pub struct BuffAbortEvent;

/// TODO: 如果激活，则abort。
/// Buff 移除事件。
#[derive(Debug, Event)]
pub struct BuffRemoveEvent;

/// 设置 buff 节流状态的事件。
#[derive(Debug, Event)]
pub struct BuffTickableEvent {
    /// 是否可执行。
    pub tickable: bool,
}

/// 处理 buff 实体添加事件：按配置数据的图类别为 buff 添加 Effect Graph。
pub fn trigger_buff_on_add(
    trigger: On<Add, Buff>,
    mut commands: Commands,
    query: Query<&BuffConfigData, With<Buff>>,
) {
    // Bevy 0.19: `On<Add, C>` Deref 到 `Add { entity }`（被添加组件的实体）；
    // `trigger.observer()` 在 0.19 返回的是 observer 实体本身（语义变更，见 TENSIONS.md）。
    let buff_entity = trigger.entity;
    match query.get(buff_entity) {
        Ok(config_data) => {
            commands.trigger(EffectGraphAddEvent {
                graph_class: config_data.graph_class.clone(),
                ability_entity: buff_entity,
            });
        }
        Err(e) => {
            warn!(
                "trigger_buff_on_add: buff config data not found for entity {:?}: {:?}",
                buff_entity, e
            );
        }
    }
}

/// 处理 [`BuffReadyEvent`]：检查开始所需/禁用状态层标签，满足则触发图 ready 执行。
#[allow(clippy::type_complexity)]
pub fn trigger_buff_ready(
    trigger: On<BuffReadyEvent>,
    state_set_query: Query<&StateLayerTagContainer>,
    mut commands: Commands,
    buff_query: Query<
        (
            &ChildOf,
            &BuffExecuteState,
            &BuffStartRequiredLayerTagContainer,
            &BuffStartDisableLayerTagContainer,
        ),
        With<Buff>,
    >,
) {
    let buff_entity = trigger.observer();

    if let Ok((parent, state, required_tag, disable_tag)) = buff_query.get(buff_entity) {
        if *state == BuffExecuteState::ToRemove {
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
                    info!("trigger_buff_ready: {:?}", buff_entity);
                    commands.trigger(EffectGraphExecEvent {
                        entry_exec_pin: EffectNodeBuffEntry::OUTPUT_EXEC_READY.into(),
                        execute_in_graph_state: Some(EffectGraphState::Inactive),
                        slot_value_map: None,
                        ability_entity: buff_entity,
                    });
                }
            }
            Err(e) => {
                warn!(
                    "trigger_buff_ready: state layer tag container not found for entity {:?}: {:?}",
                    parent.parent(),
                    e
                );
            }
        }
    }
}

/// 处理 [`BuffStartEvent`]：应用状态层标签增删并触发图 start 执行。
#[allow(clippy::type_complexity)]
pub fn trigger_buff_start(
    trigger: On<BuffStartEvent>,
    mut state_set_query: Query<&mut StateLayerTagContainer>,
    mut commands: Commands,
    buff_query: Query<
        (
            &ChildOf,
            &BuffExecuteState,
            &BuffAddedLayerTagContainer,
            &BuffRemovedLayerTagContainer,
        ),
        With<Buff>,
    >,
) {
    let buff_entity = trigger.observer();

    if let Ok((parent, state, added_tag, removed_tag)) = buff_query.get(buff_entity) {
        if *state == BuffExecuteState::ToRemove {
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
                info!("trigger_buff_start: {:?}", buff_entity);
                commands.trigger(EffectGraphExecEvent {
                    entry_exec_pin: EffectNodeBuffEntry::OUTPUT_EXEC_START.into(),
                    execute_in_graph_state: Some(EffectGraphState::Inactive),
                    slot_value_map: None,
                    ability_entity: buff_entity,
                });
            }
            Err(e) => {
                warn!(
                    "trigger_buff_start: state layer tag container not found for entity {:?}: {:?}",
                    parent.parent(),
                    e
                );
            }
        }
    }
}

/// 处理 [`BuffRemoveEvent`]：标记 buff 待移除并触发图移除事件。
pub fn trigger_buff_remove(
    trigger: On<BuffRemoveEvent>,
    mut commands: Commands,
    mut buff_query: Query<&mut BuffExecuteState, With<Buff>>,
) {
    let buff_entity = trigger.observer();
    if let Ok(mut state) = buff_query.get_mut(buff_entity) {
        info!("trigger_buff_remove: {:?}", buff_entity);
        *state = BuffExecuteState::ToRemove;
        commands.trigger(EffectGraphRemoveEvent {
            ability_entity: buff_entity,
        });
    }
}

/// 处理 [`BuffAbortEvent`]：检查中断所需/禁用状态层标签，满足则触发图 abort 执行。
pub fn trigger_buff_abort(
    trigger: On<BuffAbortEvent>,
    mut commands: Commands,
    state_set_query: Query<&StateLayerTagContainer>,
    mut buff_query: Query<
        (
            &ChildOf,
            &BuffExecuteState,
            &BuffAbortRequiredLayerTagContainer,
            &BuffAbortDisableLayerTagContainer,
        ),
        With<Buff>,
    >,
) {
    let buff_entity = trigger.observer();
    if let Ok((parent, state, required_tag, disable_tag)) = buff_query.get_mut(buff_entity) {
        if *state == BuffExecuteState::ToRemove {
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
                    info!("trigger_buff_abort: {:?}", buff_entity);
                    commands.trigger(EffectGraphExecEvent {
                        entry_exec_pin: EffectNodeBuffEntry::OUTPUT_EXEC_ABORT.into(),
                        execute_in_graph_state: Some(EffectGraphState::Inactive),
                        slot_value_map: None,
                        ability_entity: buff_entity,
                    });
                }
            }
            Err(e) => {
                warn!(
                    "trigger_buff_abort: state layer tag container not found for entity {:?}: {:?}",
                    parent.parent(),
                    e
                );
            }
        }
    }
}

/// 处理 [`BuffTickableEvent`]：透传给 buff 下的图实例节流事件。
pub fn trigger_buff_tickable(
    trigger: On<BuffTickableEvent>,
    mut commands: Commands,
    mut buff_query: Query<&BuffExecuteState, With<Buff>>,
) {
    let buff_entity = trigger.observer();
    if let Ok(state) = buff_query.get_mut(buff_entity) {
        if *state == BuffExecuteState::ToRemove {
            return;
        }

        info!("trigger_buff_abort: {:?}", buff_entity);
        commands.trigger(EffectGraphTickableEvent {
            tickable: trigger.event().tickable,
            ability_entity: buff_entity,
        });
    }
}
