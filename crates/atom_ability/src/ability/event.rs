//! 技能事件与 observer：技能的 ready/start/remove/abort/tickable 生命周期。

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
    bundle::AbilityConfigData,
    comp::{Ability, AbilityExecuteState},
    layertag::tag::{
        AbilityAbortDisableLayerTagContainer, AbilityAbortRequiredLayerTagContainer,
        AbilityAddedLayerTagContainer, AbilityRemovedLayerTagContainer,
        AbilityStartDisableLayerTagContainer, AbilityStartRequiredLayerTagContainer,
    },
};

use crate::graph::event::EffectGraphAddEvent;

/// 技能就绪事件：技能触发 ready 检查并尝试执行入口图。
#[derive(Debug, Event)]
pub struct AbilityReadyEvent;

/// 技能开始事件。
#[derive(Debug, EntityEvent)]
pub struct AbilityStartEvent {
    /// 技能实体（事件目标）。
    #[event_target]
    pub ability_entity: Entity,
}

// 需要后续处理，等待技能执行完毕。
// TODO: 如果处于激活状态下，需要触发中断事件。
/// 技能移除事件：等待技能执行完毕后移除。
#[derive(Debug, EntityEvent)]
pub struct AbilityRemoveEvent {
    /// 技能实体（事件目标）。
    #[event_target]
    pub ability_entity: Entity,
}

// 强制中断技能。
/// 技能强制中断事件。
#[derive(Debug, EntityEvent)]
pub struct AbilityAbortEvent {
    /// 技能实体（事件目标）。
    #[event_target]
    pub ability_entity: Entity,
}

/// 设置技能节流状态的事件。
#[derive(Debug, EntityEvent)]
pub struct AbilityTickableEvent {
    /// 是否可执行。
    pub tickable: bool,
    /// 技能实体（事件目标）。
    #[event_target]
    pub ability_entity: Entity,
}

/// 处理 [`AbilityReadyEvent`]：检查开始所需/禁用状态层标签，满足则触发图 ready 执行。
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

/// 处理 [`AbilityStartEvent`]：应用状态层标签增删并触发图 start 执行。
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

/// 处理 [`AbilityRemoveEvent`]：标记技能待移除并触发图移除事件。
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

/// 处理 [`AbilityAbortEvent`]：检查中断所需/禁用状态层标签，满足则触发图 abort 执行。
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

/// 处理 [`AbilityTickableEvent`]：透传给技能下的图实例节流事件。
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
/// 处理技能实体添加事件：按配置数据的图类别为技能添加 Effect Graph。
pub fn trigger_ability_add(
    trigger: On<Add, Ability>,
    mut commands: Commands,
    query: Query<&AbilityConfigData, With<Ability>>,
) {
    // Bevy 0.19: `On<Add, C>` Deref 到 `Add { entity }`（被添加组件的实体）；
    // `trigger.observer()` 在 0.19 返回的是 observer 实体本身（语义变更，见 TENSIONS.md）。
    let ability_entity = trigger.entity;
    match query.get(ability_entity) {
        Ok(config_data) => {
            commands.trigger(EffectGraphAddEvent {
                graph_class: config_data.graph_class.clone(),
                ability_entity,
            });
        }
        Err(e) => {
            warn!(
                "trigger_ability_add: ability config data not found for entity {:?}: {:?}",
                ability_entity, e
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ability::layertag::tag::AbilityLayerTagContainerRevert;
    use crate::graph::event::{
        EffectGraphAddEvent, EffectGraphExecEvent, EffectGraphRemoveEvent, EffectGraphTickableEvent,
    };
    use crate::stateset::StateLayerTagRegistry;
    use atom_datatables::effect::{Ability as AbilityRowData, AbilityType};
    use atom_layertag::container_op::LayerTagContainer;
    use atom_layertag::count_container::CountLayerTagContainer;
    use bevy::MinimalPlugins;
    use std::sync::{Arc, Mutex};

    fn tag_container(raw: &str) -> CountLayerTagContainer {
        let mut registry = StateLayerTagRegistry::default();
        registry.0.register_raw(raw);
        let layertag = registry
            .0
            .request_from_raw(raw)
            .expect("已注册标签必须可取");
        let mut container = CountLayerTagContainer::default();
        container.add_layertag(layertag);
        container
    }

    /// 注册一个只触发一次指定事件的系统。
    fn trigger_once<C>(app: &mut App, trigger_fn: C)
    where
        C: FnOnce(Commands) + Send + Sync + 'static,
    {
        let mut trigger_fn = Some(trigger_fn);
        app.add_systems(Update, move |commands: Commands, mut fired: Local<bool>| {
            if *fired {
                return;
            }
            *fired = true;
            if let Some(trigger) = trigger_fn.take() {
                trigger(commands);
            }
        });
    }

    /// 观察者：记录触发到的 EffectGraphExecEvent 的入口执行口名。
    fn record_exec_events(app: &mut App) -> Arc<Mutex<Vec<&'static str>>> {
        let received = Arc::new(Mutex::new(Vec::<&'static str>::new()));
        let received_obs = received.clone();
        app.add_observer(move |trigger: On<EffectGraphExecEvent>| {
            received_obs
                .lock()
                .expect("锁被占用")
                .push(trigger.event().entry_exec_pin.name);
        });
        received
    }

    // ===== 事件结构体 =====

    #[test]
    fn ability_events_construct() {
        let _ready = AbilityReadyEvent;

        let start = AbilityStartEvent {
            ability_entity: Entity::from_bits(1),
        };
        assert_eq!(start.ability_entity, Entity::from_bits(1));

        let remove = AbilityRemoveEvent {
            ability_entity: Entity::from_bits(2),
        };
        assert_eq!(remove.ability_entity, Entity::from_bits(2));

        let abort = AbilityAbortEvent {
            ability_entity: Entity::from_bits(3),
        };
        assert_eq!(abort.ability_entity, Entity::from_bits(3));

        let tickable = AbilityTickableEvent {
            tickable: true,
            ability_entity: Entity::from_bits(4),
        };
        assert!(tickable.tickable);
        assert_eq!(tickable.ability_entity, Entity::from_bits(4));
    }

    // ===== trigger_ability_ready =====

    #[test]
    fn ready_triggers_exec_when_tags_met() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let world = app.world_mut();
        let parent = world
            .spawn(StateLayerTagContainer(tag_container("state.a")))
            .id();
        let ability = world
            .spawn((
                Ability,
                AbilityExecuteState::Inactive,
                AbilityStartRequiredLayerTagContainer(tag_container("state.a")),
                AbilityStartDisableLayerTagContainer(CountLayerTagContainer::default()),
                Observer::new(trigger_ability_ready),
            ))
            .set_parent_in_place(parent)
            .id();

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(AbilityReadyEvent);
        });

        app.update();

        assert_eq!(
            received.lock().expect("锁被占用").as_slice(),
            &[EffectNodeAbilityEntry::OUTPUT_EXEC_READY],
            "条件满足时必须触发 ready 执行"
        );
        let _ = ability;
    }

    #[test]
    fn ready_skips_to_remove_ability() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let world = app.world_mut();
        let parent = world
            .spawn(StateLayerTagContainer(tag_container("state.a")))
            .id();
        let ability = world
            .spawn((
                Ability,
                AbilityExecuteState::ToRemove,
                AbilityStartRequiredLayerTagContainer(tag_container("state.a")),
                AbilityStartDisableLayerTagContainer(CountLayerTagContainer::default()),
                Observer::new(trigger_ability_ready),
            ))
            .set_parent_in_place(parent)
            .id();

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(AbilityReadyEvent);
        });

        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "ToRemove 技能不得触发 ready"
        );
        let _ = ability;
    }

    #[test]
    fn ready_skips_when_required_tag_missing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let world = app.world_mut();
        let parent = world
            .spawn(StateLayerTagContainer(tag_container("state.a")))
            .id();
        let ability = world
            .spawn((
                Ability,
                AbilityExecuteState::Inactive,
                AbilityStartRequiredLayerTagContainer(tag_container("state.missing")),
                AbilityStartDisableLayerTagContainer(CountLayerTagContainer::default()),
                Observer::new(trigger_ability_ready),
            ))
            .set_parent_in_place(parent)
            .id();

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(AbilityReadyEvent);
        });

        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "所需标签缺失时不得触发 ready"
        );
        let _ = ability;
    }

    #[test]
    fn ready_skips_when_disable_tag_present() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let world = app.world_mut();
        let parent = world
            .spawn(StateLayerTagContainer(tag_container("state.a")))
            .id();
        let ability = world
            .spawn((
                Ability,
                AbilityExecuteState::Inactive,
                AbilityStartRequiredLayerTagContainer(CountLayerTagContainer::default()),
                AbilityStartDisableLayerTagContainer(tag_container("state.a")),
                Observer::new(trigger_ability_ready),
            ))
            .set_parent_in_place(parent)
            .id();

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(AbilityReadyEvent);
        });

        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "存在禁用标签时不得触发 ready"
        );
        let _ = ability;
    }

    #[test]
    fn ready_skips_when_parent_container_missing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let world = app.world_mut();
        let parent = world.spawn_empty().id();
        let ability = world
            .spawn((
                Ability,
                AbilityExecuteState::Inactive,
                AbilityStartRequiredLayerTagContainer(CountLayerTagContainer::default()),
                AbilityStartDisableLayerTagContainer(CountLayerTagContainer::default()),
                Observer::new(trigger_ability_ready),
            ))
            .set_parent_in_place(parent)
            .id();

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(AbilityReadyEvent);
        });

        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "父容器缺失时走 warn 分支不执行"
        );
        let _ = ability;
    }

    // ===== trigger_ability_start =====

    #[test]
    fn start_applies_tags_and_triggers_exec() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let world = app.world_mut();
        let parent = world
            .spawn(StateLayerTagContainer(tag_container("state.existing")))
            .id();
        let ability = world
            .spawn((
                Ability,
                AbilityExecuteState::Inactive,
                AbilityAddedLayerTagContainer {
                    layer_tag_container: tag_container("state.added"),
                    revert: AbilityLayerTagContainerRevert::No,
                },
                AbilityRemovedLayerTagContainer {
                    layer_tag_container: tag_container("state.existing"),
                    revert: AbilityLayerTagContainerRevert::No,
                },
                Observer::new(trigger_ability_start),
            ))
            .set_parent_in_place(parent)
            .id();

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(AbilityStartEvent {
                ability_entity: ability,
            });
        });
        app.update();

        assert_eq!(
            received.lock().expect("锁被占用").as_slice(),
            &[EffectNodeAbilityEntry::OUTPUT_EXEC_START],
            "start 必须触发 start 执行"
        );

        let world = app.world();
        let container = world
            .entity(parent)
            .get::<StateLayerTagContainer>()
            .expect("父实体应有状态层容器");
        let tags = container
            .0
            .iter_layertag()
            .map(|t| t.raw_layertag())
            .collect::<Vec<_>>();
        assert!(
            tags.contains(&"state.added".to_string()),
            "added 标签应被应用"
        );
        assert!(
            !tags.contains(&"state.existing".to_string()),
            "removed 标签应被移除"
        );
    }

    #[test]
    fn start_skips_to_remove_ability() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let world = app.world_mut();
        let parent = world
            .spawn(StateLayerTagContainer(CountLayerTagContainer::default()))
            .id();
        let ability = world
            .spawn((
                Ability,
                AbilityExecuteState::ToRemove,
                AbilityAddedLayerTagContainer::default(),
                AbilityRemovedLayerTagContainer::default(),
                Observer::new(trigger_ability_start),
            ))
            .set_parent_in_place(parent)
            .id();

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(AbilityStartEvent {
                ability_entity: ability,
            });
        });
        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "ToRemove 技能不得触发 start"
        );
    }

    // ===== trigger_ability_remove =====

    #[test]
    fn remove_marks_to_remove_and_triggers_graph_remove() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let removed = Arc::new(Mutex::new(Vec::<Entity>::new()));
        let removed_obs = removed.clone();
        app.add_observer(move |trigger: On<EffectGraphRemoveEvent>| {
            removed_obs
                .lock()
                .expect("锁被占用")
                .push(trigger.event().ability_entity);
        });

        let world = app.world_mut();
        let parent = world
            .spawn(StateLayerTagContainer(CountLayerTagContainer::default()))
            .id();
        let ability = world
            .spawn((
                Ability,
                AbilityExecuteState::Inactive,
                Observer::new(trigger_ability_remove),
            ))
            .set_parent_in_place(parent)
            .id();

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(AbilityRemoveEvent {
                ability_entity: ability,
            });
        });
        app.update();

        let world = app.world();
        let state = world
            .entity(ability)
            .get::<AbilityExecuteState>()
            .expect("技能执行状态应存在");
        assert_eq!(*state, AbilityExecuteState::ToRemove);
        assert_eq!(
            removed.lock().expect("锁被占用").as_slice(),
            &[ability],
            "remove 必须触发图移除事件"
        );
    }

    // ===== trigger_ability_abort =====

    #[test]
    fn abort_triggers_exec_when_tags_met() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let world = app.world_mut();
        let parent = world
            .spawn(StateLayerTagContainer(tag_container("state.a")))
            .id();
        let ability = world
            .spawn((
                Ability,
                AbilityExecuteState::Inactive,
                AbilityAbortRequiredLayerTagContainer(tag_container("state.a")),
                AbilityAbortDisableLayerTagContainer(CountLayerTagContainer::default()),
                Observer::new(trigger_ability_abort),
            ))
            .set_parent_in_place(parent)
            .id();

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(AbilityAbortEvent {
                ability_entity: ability,
            });
        });
        app.update();

        assert_eq!(
            received.lock().expect("锁被占用").as_slice(),
            &[EffectNodeAbilityEntry::OUTPUT_EXEC_ABORT],
            "条件满足时必须触发 abort 执行"
        );
    }

    #[test]
    fn abort_skips_when_required_tag_missing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let world = app.world_mut();
        let parent = world
            .spawn(StateLayerTagContainer(tag_container("state.a")))
            .id();
        let ability = world
            .spawn((
                Ability,
                AbilityExecuteState::Inactive,
                AbilityAbortRequiredLayerTagContainer(tag_container("state.missing")),
                AbilityAbortDisableLayerTagContainer(CountLayerTagContainer::default()),
                Observer::new(trigger_ability_abort),
            ))
            .set_parent_in_place(parent)
            .id();

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(AbilityAbortEvent {
                ability_entity: ability,
            });
        });
        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "所需标签缺失时不得触发 abort"
        );
    }

    #[test]
    fn abort_skips_to_remove_ability() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let world = app.world_mut();
        let parent = world
            .spawn(StateLayerTagContainer(CountLayerTagContainer::default()))
            .id();
        let ability = world
            .spawn((
                Ability,
                AbilityExecuteState::ToRemove,
                AbilityAbortRequiredLayerTagContainer(CountLayerTagContainer::default()),
                AbilityAbortDisableLayerTagContainer(CountLayerTagContainer::default()),
                Observer::new(trigger_ability_abort),
            ))
            .set_parent_in_place(parent)
            .id();

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(AbilityAbortEvent {
                ability_entity: ability,
            });
        });
        app.update();
    }

    // ===== trigger_ability_tickable =====

    #[test]
    fn tickable_forwards_to_graph_tickable_event() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let received = Arc::new(Mutex::new(Vec::<bool>::new()));
        let received_obs = received.clone();
        app.add_observer(move |trigger: On<EffectGraphTickableEvent>| {
            received_obs
                .lock()
                .expect("锁被占用")
                .push(trigger.event().tickable);
        });

        let world = app.world_mut();
        let parent = world
            .spawn(StateLayerTagContainer(CountLayerTagContainer::default()))
            .id();
        let ability = world
            .spawn((
                Ability,
                AbilityExecuteState::Inactive,
                Observer::new(trigger_ability_tickable),
            ))
            .set_parent_in_place(parent)
            .id();

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(AbilityTickableEvent {
                tickable: true,
                ability_entity: ability,
            });
        });

        app.update();

        assert_eq!(
            received.lock().expect("锁被占用").as_slice(),
            &[true],
            "tickable 必须透传给图节流事件"
        );
    }

    #[test]
    fn tickable_skips_to_remove_ability() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let received = Arc::new(Mutex::new(Vec::<bool>::new()));
        let received_obs = received.clone();
        app.add_observer(move |trigger: On<EffectGraphTickableEvent>| {
            received_obs
                .lock()
                .expect("锁被占用")
                .push(trigger.event().tickable);
        });

        let world = app.world_mut();
        let parent = world
            .spawn(StateLayerTagContainer(CountLayerTagContainer::default()))
            .id();
        let ability = world
            .spawn((
                Ability,
                AbilityExecuteState::ToRemove,
                Observer::new(trigger_ability_tickable),
            ))
            .set_parent_in_place(parent)
            .id();

        let ability_entity = ability;
        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(AbilityTickableEvent {
                tickable: false,
                ability_entity,
            });
        });

        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "ToRemove 技能不得透传 tickable"
        );
    }

    // ===== trigger_ability_add =====

    fn ability_row_data(graph_class: &str) -> Arc<AbilityRowData> {
        Arc::new(AbilityRowData {
            id: 1,
            name: "test".to_string(),
            desc: "".to_string(),
            graph_class: graph_class.to_string(),
            activation_type: AbilityType::Active,
            cd: 1.0,
            start_required_layertags: vec![],
            start_disabled_layertags: vec![],
            start_added_layertags: vec![],
            start_removed_layertags: vec![],
            abort_required_layertags: vec![],
            abort_disabled_layertags: vec![],
        })
    }

    #[test]
    fn add_triggers_graph_add_event() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let received = Arc::new(Mutex::new(Vec::<String>::new()));
        let received_obs = received.clone();
        app.add_observer(move |trigger: On<EffectGraphAddEvent>| {
            received_obs
                .lock()
                .expect("锁被占用")
                .push(trigger.event().graph_class.clone());
        });

        let world = app.world_mut();
        let ability = world
            .spawn((
                Observer::new(trigger_ability_add),
                TbAbilityRow {
                    key: 1,
                    data: Some(ability_row_data("fireball")),
                },
            ))
            .id();
        world.entity_mut(ability).insert(Ability);

        app.update();
        app.update();

        assert_eq!(
            received.lock().expect("锁被占用").as_slice(),
            &["fireball".to_string()],
            "add 必须按数据表图类别触发图添加事件"
        );
        let _ = ability;
    }

    #[test]
    fn add_without_row_data_does_nothing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let received = Arc::new(Mutex::new(Vec::<String>::new()));
        let received_obs = received.clone();
        app.add_observer(move |trigger: On<EffectGraphAddEvent>| {
            received_obs
                .lock()
                .expect("锁被占用")
                .push(trigger.event().graph_class.clone());
        });

        let world = app.world_mut();
        let ability = world
            .spawn((
                Observer::new(trigger_ability_add),
                TbAbilityRow { key: 2, data: None },
            ))
            .id();
        world.entity_mut(ability).insert(Ability);

        app.update();
        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "data 为 None 时不得触发图添加事件"
        );
        let _ = ability;
    }

    #[test]
    fn add_without_row_component_warns() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let world = app.world_mut();
        let ability = world.spawn(Observer::new(trigger_ability_add)).id();
        world.entity_mut(ability).insert(Ability);

        app.update();
        app.update();

        let _ = ability;
    }
}
