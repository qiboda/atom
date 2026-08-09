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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buff::layertag::tag::BuffLayerTagContainerRevert;
    use crate::buff::state::BuffExecuteState;
    use crate::graph::event::{
        EffectGraphAddEvent, EffectGraphExecEvent, EffectGraphRemoveEvent, EffectGraphTickableEvent,
    };
    use crate::stateset::{StateLayerTagContainer, StateLayerTagRegistry};
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

    /// 构造 buff 实体（observer 随实体注册，父实体持有状态层容器）。
    fn spawn_buff_entity(
        app: &mut App,
        parent_tags: CountLayerTagContainer,
        state: BuffExecuteState,
    ) -> (Entity, Entity) {
        let world = app.world_mut();
        let parent = world.spawn(StateLayerTagContainer(parent_tags)).id();
        let buff = world
            .spawn((
                Buff,
                state,
                BuffStartRequiredLayerTagContainer(CountLayerTagContainer::default()),
                BuffStartDisableLayerTagContainer(CountLayerTagContainer::default()),
                BuffAddedLayerTagContainer::default(),
                BuffRemovedLayerTagContainer::default(),
                BuffAbortRequiredLayerTagContainer(CountLayerTagContainer::default()),
                BuffAbortDisableLayerTagContainer(CountLayerTagContainer::default()),
            ))
            .set_parent_in_place(parent)
            .id();
        (parent, buff)
    }

    // ===== 事件结构体 =====

    #[test]
    fn buff_events_construct() {
        let add = BuffAddEvent {
            owner_entity: Entity::from_bits(1),
            buff_id: 7,
        };
        assert_eq!(add.owner_entity, Entity::from_bits(1));
        assert_eq!(add.buff_id, 7);

        let _ready = BuffReadyEvent;
        let _start = BuffStartEvent;
        let _abort = BuffAbortEvent;
        let _remove = BuffRemoveEvent;

        let tickable = BuffTickableEvent { tickable: true };
        assert!(tickable.tickable);
    }

    // ===== trigger_buff_on_add =====

    fn buff_config_data(graph_class: &str) -> BuffConfigData {
        BuffConfigData {
            graph_class: graph_class.to_string(),
        }
    }

    #[test]
    fn on_add_triggers_graph_add_event() {
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

        {
            let world = app.world_mut();
            let buff = world
                .spawn((
                    Observer::new(trigger_buff_on_add),
                    buff_config_data("buff_graph"),
                ))
                .id();
            world.entity_mut(buff).insert(Buff);
        }

        app.update();
        app.update();

        assert_eq!(
            received.lock().expect("锁被占用").as_slice(),
            &["buff_graph".to_string()],
            "on_add 必须按配置数据图类别触发图添加事件"
        );
    }

    #[test]
    fn on_add_without_config_data_does_nothing() {
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

        {
            let world = app.world_mut();
            let buff = world.spawn(Observer::new(trigger_buff_on_add)).id();
            world.entity_mut(buff).insert(Buff);
        }

        app.update();
        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "无配置数据时不得触发图添加事件"
        );
    }

    #[test]
    fn on_add_without_row_component_warns() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        {
            let world = app.world_mut();
            let buff = world.spawn(Observer::new(trigger_buff_on_add)).id();
            world.entity_mut(buff).insert(Buff);
        }

        app.update();
        app.update();
    }

    // ===== trigger_buff_ready =====

    #[test]
    fn ready_triggers_exec_when_tags_met() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let (parent, buff) = spawn_buff_entity(
            &mut app,
            tag_container("state.a"),
            BuffExecuteState::Inactive,
        );
        app.world_mut()
            .entity_mut(buff)
            .insert(Observer::new(trigger_buff_ready));
        app.world_mut()
            .entity_mut(buff)
            .insert(BuffStartRequiredLayerTagContainer(tag_container("state.a")));
        app.world_mut()
            .entity_mut(buff)
            .insert(BuffStartDisableLayerTagContainer(
                CountLayerTagContainer::default(),
            ));

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(BuffReadyEvent);
        });

        app.update();

        assert_eq!(
            received.lock().expect("锁被占用").as_slice(),
            &[EffectNodeBuffEntry::OUTPUT_EXEC_READY],
            "条件满足时必须触发 ready 执行"
        );
        let _ = parent;
    }

    #[test]
    fn ready_skips_to_remove_buff() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let (_parent, buff) = spawn_buff_entity(
            &mut app,
            tag_container("state.a"),
            BuffExecuteState::ToRemove,
        );
        app.world_mut()
            .entity_mut(buff)
            .insert(Observer::new(trigger_buff_ready));

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(BuffReadyEvent);
        });

        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "ToRemove buff 不得触发 ready"
        );
    }

    #[test]
    fn ready_skips_when_required_tag_missing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let (_parent, buff) = spawn_buff_entity(
            &mut app,
            tag_container("state.a"),
            BuffExecuteState::Inactive,
        );
        app.world_mut()
            .entity_mut(buff)
            .insert(Observer::new(trigger_buff_ready));
        app.world_mut()
            .entity_mut(buff)
            .insert(BuffStartRequiredLayerTagContainer(tag_container(
                "state.missing",
            )));
        app.world_mut()
            .entity_mut(buff)
            .insert(BuffStartDisableLayerTagContainer(
                CountLayerTagContainer::default(),
            ));

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(BuffReadyEvent);
        });

        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "所需标签缺失时不得触发 ready"
        );
    }

    #[test]
    fn ready_skips_when_disable_tag_present() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let (_parent, buff) = spawn_buff_entity(
            &mut app,
            tag_container("state.a"),
            BuffExecuteState::Inactive,
        );
        app.world_mut()
            .entity_mut(buff)
            .insert(Observer::new(trigger_buff_ready));
        app.world_mut()
            .entity_mut(buff)
            .insert(BuffStartRequiredLayerTagContainer(
                CountLayerTagContainer::default(),
            ));
        app.world_mut()
            .entity_mut(buff)
            .insert(BuffStartDisableLayerTagContainer(tag_container("state.a")));

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(BuffReadyEvent);
        });

        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "存在禁用标签时不得触发 ready"
        );
    }

    // ===== trigger_buff_start =====

    #[test]
    fn start_applies_tags_and_triggers_exec() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let (parent, buff) = spawn_buff_entity(
            &mut app,
            tag_container("state.existing"),
            BuffExecuteState::Inactive,
        );
        app.world_mut()
            .entity_mut(buff)
            .insert(Observer::new(trigger_buff_start));
        app.world_mut()
            .entity_mut(buff)
            .insert(BuffAddedLayerTagContainer {
                layer_tag_container: tag_container("state.added"),
                revert: BuffLayerTagContainerRevert::No,
            });
        app.world_mut()
            .entity_mut(buff)
            .insert(BuffRemovedLayerTagContainer {
                layer_tag_container: tag_container("state.existing"),
                revert: BuffLayerTagContainerRevert::No,
            });

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(BuffStartEvent);
        });

        app.update();

        assert_eq!(
            received.lock().expect("锁被占用").as_slice(),
            &[EffectNodeBuffEntry::OUTPUT_EXEC_START],
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
    fn start_skips_to_remove_buff() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let (_parent, buff) = spawn_buff_entity(
            &mut app,
            CountLayerTagContainer::default(),
            BuffExecuteState::ToRemove,
        );
        app.world_mut()
            .entity_mut(buff)
            .insert(Observer::new(trigger_buff_start));

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(BuffStartEvent);
        });

        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "ToRemove buff 不得触发 start"
        );
    }

    // ===== trigger_buff_remove =====

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

        let (_parent, buff) = spawn_buff_entity(
            &mut app,
            CountLayerTagContainer::default(),
            BuffExecuteState::Inactive,
        );
        app.world_mut()
            .entity_mut(buff)
            .insert(Observer::new(trigger_buff_remove));

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(BuffRemoveEvent);
        });

        app.update();

        let world = app.world();
        let state = world
            .entity(buff)
            .get::<BuffExecuteState>()
            .expect("buff 执行状态应存在");
        assert_eq!(*state, BuffExecuteState::ToRemove);
        assert_eq!(
            removed.lock().expect("锁被占用").as_slice(),
            &[buff],
            "remove 必须触发图移除事件"
        );
    }

    // ===== trigger_buff_abort =====

    #[test]
    fn abort_triggers_exec_when_tags_met() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let (_parent, buff) = spawn_buff_entity(
            &mut app,
            tag_container("state.a"),
            BuffExecuteState::Inactive,
        );
        app.world_mut()
            .entity_mut(buff)
            .insert(Observer::new(trigger_buff_abort));
        app.world_mut()
            .entity_mut(buff)
            .insert(BuffAbortRequiredLayerTagContainer(tag_container("state.a")));
        app.world_mut()
            .entity_mut(buff)
            .insert(BuffAbortDisableLayerTagContainer(
                CountLayerTagContainer::default(),
            ));

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(BuffAbortEvent);
        });

        app.update();

        assert_eq!(
            received.lock().expect("锁被占用").as_slice(),
            &[EffectNodeBuffEntry::OUTPUT_EXEC_ABORT],
            "条件满足时必须触发 abort 执行"
        );
    }

    #[test]
    fn abort_skips_when_required_tag_missing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let received = record_exec_events(&mut app);

        let (_parent, buff) = spawn_buff_entity(
            &mut app,
            tag_container("state.a"),
            BuffExecuteState::Inactive,
        );
        app.world_mut()
            .entity_mut(buff)
            .insert(Observer::new(trigger_buff_abort));
        app.world_mut()
            .entity_mut(buff)
            .insert(BuffAbortRequiredLayerTagContainer(tag_container(
                "state.missing",
            )));
        app.world_mut()
            .entity_mut(buff)
            .insert(BuffAbortDisableLayerTagContainer(
                CountLayerTagContainer::default(),
            ));

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(BuffAbortEvent);
        });

        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "所需标签缺失时不得触发 abort"
        );
    }

    #[test]
    fn abort_skips_to_remove_buff() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let (_parent, buff) = spawn_buff_entity(
            &mut app,
            CountLayerTagContainer::default(),
            BuffExecuteState::ToRemove,
        );
        app.world_mut()
            .entity_mut(buff)
            .insert(Observer::new(trigger_buff_abort));

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(BuffAbortEvent);
        });

        app.update();
    }

    // ===== trigger_buff_tickable =====

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

        let (_parent, buff) = spawn_buff_entity(
            &mut app,
            CountLayerTagContainer::default(),
            BuffExecuteState::Inactive,
        );
        app.world_mut()
            .entity_mut(buff)
            .insert(Observer::new(trigger_buff_tickable));

        trigger_once(&mut app, |mut commands: Commands| {
            commands.trigger(BuffTickableEvent { tickable: true });
        });

        app.update();

        assert_eq!(
            received.lock().expect("锁被占用").as_slice(),
            &[true],
            "tickable 必须透传给图节流事件"
        );
    }

    #[test]
    fn tickable_skips_to_remove_buff() {
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

        let (_parent, buff) = spawn_buff_entity(
            &mut app,
            CountLayerTagContainer::default(),
            BuffExecuteState::ToRemove,
        );
        app.world_mut()
            .entity_mut(buff)
            .insert(Observer::new(trigger_buff_tickable));

        trigger_once(&mut app, |mut commands: Commands| {
            commands.trigger(BuffTickableEvent { tickable: false });
        });

        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "ToRemove buff 不得透传 tickable"
        );
    }
}
