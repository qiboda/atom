//! Effect Graph 对外事件与 observer：图实例的添加、执行、克隆、移除与节流。

use bevy::{ecs::entity::EntityHashMap, platform::collections::HashMap, prelude::*};

use crate::graph::pin::EffectNodeSlotPin;

use super::{
    EffectGraphOwner,
    blackboard::EffectValue,
    context::{EffectGraphContext, GraphRef, InstantEffectNodeMap},
    executor::EffectGraphExecutor,
    graph_map::{EffectGraphBuilderMap, EffectGraphMap, GraphClass},
    node::pin::{EffectNodeExec, EffectNodeSlot},
    pin::EffectNodeExecPin,
    state::{EffectGraphState, EffectGraphTickState},
};

/// 执行图内单个节点的事件（触发 `run a graph node`）。
#[derive(Debug, Event)]
pub struct EffectNodeExecEvent {
    /// 要执行的输入执行口。
    pub input_exec_pin: EffectNodeExecPin,
}

/// 图克隆开始事件：将模板图克隆为新的图实例。
#[derive(Debug, EntityEvent, Clone, Copy)]
pub struct CloneEffectGraphStartEvent {
    /// 新图实例实体（事件目标）。
    #[event_target]
    pub new_graph_instance: Entity,
    /// 源图（模板）引用。
    pub graph_ref: GraphRef,
    /// 新图实例的根实体。
    pub destination_entity: Entity,
}

impl Default for CloneEffectGraphStartEvent {
    fn default() -> Self {
        Self {
            new_graph_instance: Entity::PLACEHOLDER,
            graph_ref: GraphRef::new(Entity::PLACEHOLDER),
            destination_entity: Entity::PLACEHOLDER,
        }
    }
}

/// 图克隆完成事件：携带新旧实体映射，用于重写新图上下文中的实体引用。
#[derive(Debug, EntityEvent)]
pub struct CloneEffectGraphEndEvent {
    /// 新图的根实体。
    pub destination_root_entity: Entity,
    /// 旧实体 → 新实体映射。
    pub old_new_entities: EntityHashMap<Entity>,
    /// 新图实例实体（事件目标）。
    #[event_target]
    pub new_graph_instance: Entity,
}

// 添加一个EffectGraph
/// 为技能实体添加 Effect Graph 实例的事件。
#[derive(Debug, EntityEvent, Clone)]
pub struct EffectGraphAddEvent {
    /// 技能实体（事件目标）。
    #[event_target]
    pub ability_entity: Entity,
    /// 图类别，用于查找图模板。
    pub graph_class: GraphClass,
}

/// 移除技能实体上的 Effect Graph 的事件。
#[derive(Debug, EntityEvent, Clone, Copy)]
pub struct EffectGraphRemoveEvent {
    /// 技能实体（事件目标）。
    #[event_target]
    pub ability_entity: Entity,
}

/// 执行技能 Effect Graph 的事件。
#[derive(Debug, EntityEvent, Clone)]
pub struct EffectGraphExecEvent {
    /// 技能实体（事件目标）。
    #[event_target]
    pub ability_entity: Entity,
    /// 入口执行口（`"ready"` 时按技能实例复用规则执行）。
    pub entry_exec_pin: EffectNodeExec,
    /// 仅当图处于该状态时才执行（`None` 表示无条件执行）。
    pub execute_in_graph_state: Option<EffectGraphState>,
    /// 入口节点输入槽的初始值。
    pub slot_value_map: Option<HashMap<EffectNodeSlot, EffectValue>>,
}

/// 设置技能 Effect Graph 节流状态的事件。
#[derive(Debug, EntityEvent, Clone, Copy)]
pub struct EffectGraphTickableEvent {
    /// 是否可执行。
    pub tickable: bool,
    /// 技能实体（事件目标）。
    #[event_target]
    pub ability_entity: Entity,
}

/// 处理 [`EffectGraphTickableEvent`]：更新技能下所有图实例的节流状态。
pub fn trigger_effect_graph_tickable(
    trigger: On<EffectGraphTickableEvent>,
    graph_owner_query: Query<&Children, With<EffectGraphOwner>>,
    mut graph_query: Query<&mut EffectGraphTickState>,
) {
    let graph_owner_entity = trigger.observer();
    let Ok(children) = graph_owner_query.get(graph_owner_entity) else {
        return;
    };

    let event = trigger.event();
    for child in children {
        if let Ok(mut state) = graph_query.get_mut(*child) {
            if event.tickable {
                *state = EffectGraphTickState::Ticked;
            } else {
                *state = EffectGraphTickState::Paused;
            }
            info!(
                "trigger effect graph tickable: {:?} => {:?} : {:?}",
                graph_owner_entity, child, state
            );
        }
    }
}

/// 处理 [`EffectGraphRemoveEvent`]：将技能下所有图实例标记为待销毁。
pub fn trigger_effect_graph_to_remove(
    trigger: On<EffectGraphRemoveEvent>,
    graph_owner_query: Query<&Children, With<EffectGraphOwner>>,
    mut graph_query: Query<&mut EffectGraphState>,
) {
    let graph_owner_entity = trigger.observer();
    let Ok(children) = graph_owner_query.get(graph_owner_entity) else {
        return;
    };

    for child in children {
        if let Ok(mut state) = graph_query.get_mut(*child) {
            info!(
                "trigger effect graph remove: {:?} => {:?} ",
                graph_owner_entity, child
            );
            *state = EffectGraphState::ToRemove;
        }
    }
}

/// 处理 [`EffectGraphExecEvent`]：从入口节点开始执行图；技能激活中再次 ready
/// 执行时会先克隆新图实例再执行，避免复用同一实例导致状态混乱。
pub fn trigger_effect_graph_exec(
    trigger: On<EffectGraphExecEvent>,
    mut commands: Commands,
    graph_owner_query: Query<&Children, With<EffectGraphOwner>>,
    mut graph_query: Query<(
        &mut EffectGraphContext,
        &mut EffectGraphExecutor,
        &EffectGraphState,
    )>,
    instant_map: Res<InstantEffectNodeMap>,
) {
    let graph_owner_entity = trigger.observer();
    let Ok(children) = graph_owner_query.get(graph_owner_entity) else {
        return;
    };

    info!(
        "trigger effect graph exec: {:?} => {:?} ",
        graph_owner_entity,
        trigger.event()
    );

    // NOTE: Ready exec pin must use "ready" name.
    // TODO: Effect Graph state的设置也没有意义。
    // 以上有问题，如果技能需要还原，结束，中断等，多次技能使用同一个EffectGraph instance，
    // 会有问题。因为每个节点存储了多个状态，但是不知道应该还原，结束，中断哪些状态。
    if trigger.event().entry_exec_pin == "ready".into() {
        let mut clone_event = CloneEffectGraphStartEvent::default();
        let mut to_clone_graph = false;
        for child in children {
            if let Ok((context, mut _executor, state)) = graph_query.get(*child) {
                match state {
                    EffectGraphState::Inactive => {
                        to_clone_graph = false;
                        break;
                    }
                    EffectGraphState::Active => {
                        clone_event.graph_ref =
                            context.get_graph_ref().expect("graph ref must exist");
                        to_clone_graph = true;
                    }
                    EffectGraphState::ToRemove => {}
                }
            }
        }

        if to_clone_graph {
            let new_graph_instance = commands
                .spawn_empty()
                .set_parent_in_place(graph_owner_entity)
                .id();
            clone_event.destination_entity = new_graph_instance;
            clone_event.new_graph_instance = new_graph_instance;
            commands.trigger(clone_event);
            info!(
                "trigger effect graph exec to clone graph: {:?} => {:?} => {:?} ",
                graph_owner_entity,
                trigger.event(),
                clone_event
            );

            let mut event_clone = trigger.event().clone();
            event_clone.ability_entity = graph_owner_entity;
            commands.trigger(event_clone);
            return;
        }
    }

    for child in children {
        if let Ok((mut context, mut executor, state)) = graph_query.get_mut(*child) {
            let mut execute_start = || {
                info!(
                    "trigger effect graph exec: {:?} => {:?} in state {:?} ",
                    graph_owner_entity,
                    trigger.event(),
                    state
                );

                if let Some(entry_entity) = context.get_entry_node() {
                    if let Some(slot_value_map) = trigger.event().slot_value_map.as_ref() {
                        for (slot, value) in slot_value_map {
                            context.insert_output_value(
                                EffectNodeSlotPin {
                                    node_id: entry_entity.into(),
                                    slot: *slot,
                                },
                                value.clone().into(),
                            );
                        }
                    }
                    executor.start_push_output_pin(
                        EffectNodeExecPin {
                            node_id: entry_entity.into(),
                            exec: trigger.event().entry_exec_pin,
                        },
                        &context,
                        &instant_map,
                    );
                }
            };

            match trigger.event().execute_in_graph_state {
                Some(execute_state) => {
                    if *state == execute_state {
                        execute_start();
                    }
                }
                None => {
                    execute_start();
                }
            }
        }
    }
}

/// 处理 [`EffectGraphAddEvent`]：查找/构建图模板，并为技能实体克隆一个新图实例。
pub fn trigger_effect_graph_add(
    trigger: On<EffectGraphAddEvent>,
    mut commands: Commands,
    mut graph_map: ResMut<EffectGraphMap>,
    graph_builder_map: Res<EffectGraphBuilderMap>,
    mut instant_map: ResMut<InstantEffectNodeMap>,
) {
    let graph_owner_entity = trigger.observer();
    let event = trigger.event();

    let mut graph_ref = graph_map.get_graph(event.graph_class.clone());
    if graph_ref.is_none() {
        let graph_builder = graph_builder_map.get_effect_graph_builder(&event.graph_class);
        match graph_builder {
            Some(builder) => {
                let graph = builder.build(&mut commands, &mut instant_map);
                graph_ref = Some(GraphRef::new(graph));
                info!("build graph template: {:?}", graph);
                graph_map.insert_graph(
                    event.graph_class.clone(),
                    graph_ref.expect("graph ref must exist"),
                );
            }
            None => {
                error!("graph builder not found: {}", event.graph_class);
            }
        }
    }

    let Some(graph_ref) = graph_ref else {
        return;
    };

    let new_graph_entity = commands
        .spawn_empty()
        .set_parent_in_place(graph_owner_entity)
        .id();
    commands.trigger(CloneEffectGraphStartEvent {
        graph_ref,
        destination_entity: new_graph_entity,
        new_graph_instance: new_graph_entity,
    });
}

/// 处理 [`CloneEffectGraphStartEvent`]：用 Bevy 的 `EntityCloner` 递归克隆
/// 模板图实体树，完成后触发 [`CloneEffectGraphEndEvent`]。
pub fn trigger_clone_effect_graph_start(
    trigger: On<CloneEffectGraphStartEvent>,
    mut commands: Commands,
) {
    let event = trigger.event();
    assert_ne!(trigger.observer(), Entity::PLACEHOLDER);

    let graph_ref = event.graph_ref;
    let new_graph_entity = event.destination_entity;
    let source_entity = graph_ref.get_entity();

    // 使用 Bevy 内置的 EntityCloner 递归克隆 entity 树
    commands.queue(move |world: &mut World| {
        use bevy::ecs::entity::EntityCloner;

        let mut mapper = EntityHashMap::<Entity>::new();
        mapper.insert(source_entity, new_graph_entity);

        let mut cloner = EntityCloner::build_opt_out(world);
        cloner.linked_cloning(true);
        let mut cloner = cloner.finish();
        cloner.clone_entity_mapped(world, source_entity, &mut mapper);

        let mut old_new_entities = mapper;
        old_new_entities.remove(&source_entity);

        world.trigger(CloneEffectGraphEndEvent {
            destination_root_entity: new_graph_entity,
            old_new_entities,
            new_graph_instance: new_graph_entity,
        });

        info!("clone_effect_graph_start");
    });
}

/// 处理 [`CloneEffectGraphEndEvent`]：为新图实例挂载执行器，并将上下文中的
/// 实体引用从旧实体替换为新实体。
pub fn trigger_clone_effect_graph_end(
    trigger: On<CloneEffectGraphEndEvent>,
    mut commands: Commands,
    mut query: Query<&mut EffectGraphContext>,
) {
    let event = trigger.event();
    assert_ne!(trigger.observer(), Entity::PLACEHOLDER);

    commands
        .entity(trigger.observer())
        .insert(EffectGraphExecutor::default());

    info!("clone entities: {:?}", event.old_new_entities);

    match query.get_mut(trigger.observer()) {
        Ok(mut context) => {
            // info!("context: before");
            // dbg!(&context);
            context.replace_state_entities(event.old_new_entities.clone());
            // info!("context: after");
            // dbg!(&context);
        }
        Err(e) => {
            error!(
                "effect graph context not found for entity: {:?}, error: {:?}",
                trigger.observer(),
                e
            );
        }
    }

    info!("clone_effect_graph_end");
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::builder::EffectGraphBuilder;
    use crate::graph::graph_map::EffectGraphBuilderMapExt;
    use crate::graph::node::EffectNodeId;
    use crate::graph::pin::EffectNodeSlotValue;
    use bevy::MinimalPlugins;
    use std::sync::{Arc, Mutex};

    fn exec_pin(node_id: impl Into<EffectNodeId>, name: &'static str) -> EffectNodeExecPin {
        EffectNodeExecPin {
            node_id: node_id.into(),
            exec: EffectNodeExec { name },
        }
    }

    fn slot_pin(node_id: impl Into<EffectNodeId>, name: &'static str) -> EffectNodeSlotPin {
        EffectNodeSlotPin {
            node_id: node_id.into(),
            slot: EffectNodeSlot::new::<f32>(name),
        }
    }

    /// 注册一个只触发一次指定事件的系统。
    fn trigger_once<C>(app: &mut App, trigger_fn: C)
    where
        C: FnOnce(Commands) + Send + Sync + 'static,
    {
        #[derive(Resource, Default)]
        struct Fired(bool);
        app.init_resource::<Fired>();
        let mut trigger_fn = Some(trigger_fn);
        app.add_systems(
            Update,
            move |commands: Commands, mut fired: ResMut<Fired>| {
                if fired.0 {
                    return;
                }
                fired.0 = true;
                if let Some(trigger) = trigger_fn.take() {
                    trigger(commands);
                }
            },
        );
    }

    /// 创建 owner 实体（带 EffectGraphOwner）并挂载一个图实例子实体。
    fn spawn_owner_with_graph(
        app: &mut App,
        graph_state: EffectGraphState,
        entry_node: Option<Entity>,
    ) -> (Entity, Entity) {
        let world = app.world_mut();
        let owner = world.spawn(EffectGraphOwner).id();
        let mut context = EffectGraphContext::new();
        context.entry_node = entry_node;
        let graph = world
            .spawn((context, EffectGraphExecutor::default(), graph_state))
            .set_parent_in_place(owner)
            .id();
        (owner, graph)
    }

    // ===== 事件结构体 =====

    #[test]
    fn node_exec_event_constructs_with_pin() {
        let event = EffectNodeExecEvent {
            input_exec_pin: exec_pin(Entity::from_bits(1), "start"),
        };
        assert_eq!(
            event.input_exec_pin,
            exec_pin(Entity::from_bits(1), "start")
        );
    }

    #[test]
    fn clone_start_event_default_and_fields() {
        let event = CloneEffectGraphStartEvent::default();
        assert_eq!(event.new_graph_instance, Entity::PLACEHOLDER);
        assert_eq!(event.destination_entity, Entity::PLACEHOLDER);
        assert_eq!(event.graph_ref.get_entity(), Entity::PLACEHOLDER);

        let event = CloneEffectGraphStartEvent {
            new_graph_instance: Entity::from_bits(1),
            graph_ref: GraphRef::new(Entity::from_bits(2)),
            destination_entity: Entity::from_bits(3),
        };
        assert_eq!(event.new_graph_instance, Entity::from_bits(1));
        assert_eq!(event.graph_ref.get_entity(), Entity::from_bits(2));
        assert_eq!(event.destination_entity, Entity::from_bits(3));
    }

    #[test]
    fn clone_end_event_constructs_with_mapping() {
        let mut mapping = EntityHashMap::default();
        mapping.insert(Entity::from_bits(1), Entity::from_bits(2));
        let event = CloneEffectGraphEndEvent {
            destination_root_entity: Entity::from_bits(10),
            old_new_entities: mapping.clone(),
            new_graph_instance: Entity::from_bits(11),
        };
        assert_eq!(event.destination_root_entity, Entity::from_bits(10));
        assert_eq!(event.old_new_entities, mapping);
        assert_eq!(event.new_graph_instance, Entity::from_bits(11));
    }

    #[test]
    fn graph_add_remove_exec_tickable_events_construct() {
        let add = EffectGraphAddEvent {
            ability_entity: Entity::from_bits(1),
            graph_class: "fireball".to_string(),
        };
        assert_eq!(add.ability_entity, Entity::from_bits(1));
        assert_eq!(add.graph_class, "fireball");

        let remove = EffectGraphRemoveEvent {
            ability_entity: Entity::from_bits(2),
        };
        assert_eq!(remove.ability_entity, Entity::from_bits(2));

        let exec = EffectGraphExecEvent {
            ability_entity: Entity::from_bits(3),
            entry_exec_pin: EffectNodeExec { name: "ready" },
            execute_in_graph_state: Some(EffectGraphState::Inactive),
            slot_value_map: None,
        };
        assert_eq!(exec.ability_entity, Entity::from_bits(3));
        assert_eq!(exec.entry_exec_pin.name, "ready");
        assert_eq!(
            exec.execute_in_graph_state,
            Some(EffectGraphState::Inactive)
        );
        assert!(exec.slot_value_map.is_none());

        let exec_with_map = EffectGraphExecEvent {
            ability_entity: Entity::from_bits(3),
            entry_exec_pin: EffectNodeExec { name: "ready" },
            execute_in_graph_state: None,
            slot_value_map: Some(HashMap::new()),
        };
        assert!(exec_with_map.slot_value_map.is_some());

        let tickable = EffectGraphTickableEvent {
            tickable: true,
            ability_entity: Entity::from_bits(4),
        };
        assert!(tickable.tickable);
        assert_eq!(tickable.ability_entity, Entity::from_bits(4));
    }

    // ===== trigger_effect_graph_tickable =====

    #[test]
    fn tickable_event_marks_children_ticked_or_paused() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let (owner, child_a, child_b) = {
            let world = app.world_mut();
            let owner = world
                .spawn((
                    Observer::new(trigger_effect_graph_tickable),
                    EffectGraphOwner,
                ))
                .id();
            let child_a = world
                .spawn(EffectGraphTickState::Ticked)
                .set_parent_in_place(owner)
                .id();
            let child_b = world
                .spawn(EffectGraphTickState::Paused)
                .set_parent_in_place(owner)
                .id();
            (owner, child_a, child_b)
        };

        // 同一系统按帧依次触发 tickable: true → false。
        app.add_systems(
            Update,
            move |mut commands: Commands, mut step: Local<u8>| {
                match *step {
                    0 => commands.trigger(EffectGraphTickableEvent {
                        tickable: true,
                        ability_entity: owner,
                    }),
                    1 => commands.trigger(EffectGraphTickableEvent {
                        tickable: false,
                        ability_entity: owner,
                    }),
                    _ => {}
                }
                *step += 1;
            },
        );

        app.update();
        app.update();
        app.update();
        app.update();

        {
            let world = app.world();
            assert_eq!(
                *world
                    .entity(child_a)
                    .get::<EffectGraphTickState>()
                    .expect("子图节流状态应存在"),
                EffectGraphTickState::Paused
            );
            assert_eq!(
                *world
                    .entity(child_b)
                    .get::<EffectGraphTickState>()
                    .expect("子图节流状态应存在"),
                EffectGraphTickState::Paused
            );
        }
    }

    #[test]
    fn tickable_event_skips_owner_without_children() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let owner = app
            .world_mut()
            .spawn((
                Observer::new(trigger_effect_graph_tickable),
                EffectGraphOwner,
            ))
            .id();

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(EffectGraphTickableEvent {
                tickable: true,
                ability_entity: owner,
            });
        });

        app.update();
    }

    #[test]
    fn tickable_event_skips_children_without_tick_state() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let owner = {
            let world = app.world_mut();
            let owner = world
                .spawn((
                    Observer::new(trigger_effect_graph_tickable),
                    EffectGraphOwner,
                ))
                .id();
            let _plain = world.spawn_empty().set_parent_in_place(owner).id();
            owner
        };

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(EffectGraphTickableEvent {
                tickable: false,
                ability_entity: owner,
            });
        });

        app.update();
    }

    // ===== trigger_effect_graph_to_remove =====

    #[test]
    fn remove_event_marks_all_children_to_remove() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let (owner, child_a, child_b) = {
            let world = app.world_mut();
            let owner = world
                .spawn((
                    Observer::new(trigger_effect_graph_to_remove),
                    EffectGraphOwner,
                ))
                .id();
            let child_a = world
                .spawn(EffectGraphState::Inactive)
                .set_parent_in_place(owner)
                .id();
            let child_b = world
                .spawn(EffectGraphState::Active)
                .set_parent_in_place(owner)
                .id();
            (owner, child_a, child_b)
        };
        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(EffectGraphRemoveEvent {
                ability_entity: owner,
            });
        });

        app.update();

        let world = app.world();
        assert_eq!(
            *world
                .entity(child_a)
                .get::<EffectGraphState>()
                .expect("子图状态应存在"),
            EffectGraphState::ToRemove
        );
        assert_eq!(
            *world
                .entity(child_b)
                .get::<EffectGraphState>()
                .expect("子图状态应存在"),
            EffectGraphState::ToRemove
        );
    }

    #[test]
    fn remove_event_skips_owner_without_children() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let owner = app
            .world_mut()
            .spawn((
                Observer::new(trigger_effect_graph_to_remove),
                EffectGraphOwner,
            ))
            .id();

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(EffectGraphRemoveEvent {
                ability_entity: owner,
            });
        });

        app.update();
    }

    // ===== trigger_effect_graph_exec =====

    /// 构造 exec observer 测试 App：注册执行器系统与即时节点资源。
    fn exec_test_app() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            crate::graph::executor::EffectGraphExecutorPlugin,
        ));
        app.init_resource::<InstantEffectNodeMap>();
        app
    }

    #[test]
    fn exec_triggers_entry_node_chain() {
        let mut app = exec_test_app();

        let (owner, _entry_node, target, _graph) = {
            let world = app.world_mut();
            let owner = world
                .spawn((Observer::new(trigger_effect_graph_exec), EffectGraphOwner))
                .id();
            let entry_node = world.spawn_empty().id();
            let target = world.spawn_empty().id();
            let graph = world
                .spawn((
                    EffectGraphContext::new(),
                    EffectGraphExecutor::default(),
                    EffectGraphState::Inactive,
                ))
                .set_parent_in_place(owner)
                .id();
            {
                let mut context = world
                    .get_mut::<EffectGraphContext>(graph)
                    .expect("图上下文应存在");
                context.entry_node = Some(entry_node);
                context
                    .add_exec_connection(exec_pin(entry_node, "start"), &[exec_pin(target, "run")]);
            }
            (owner, entry_node, target, graph)
        };

        let received = Arc::new(Mutex::new(Vec::<EffectNodeExecPin>::new()));
        let received_obs = received.clone();
        app.add_observer(move |trigger: On<EffectNodeExecEvent>| {
            received_obs
                .lock()
                .expect("锁被占用")
                .push(trigger.event().input_exec_pin);
        });

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(EffectGraphExecEvent {
                ability_entity: owner,
                entry_exec_pin: EffectNodeExec { name: "start" },
                execute_in_graph_state: None,
                slot_value_map: None,
            });
        });

        app.update();
        app.update();

        let events = received.lock().expect("锁被占用");
        assert_eq!(
            events.as_slice(),
            &[exec_pin(target, "run")],
            "exec 必须沿入口节点连接触发目标节点"
        );
    }

    #[test]
    fn exec_respects_execute_in_graph_state_filter() {
        let mut app = exec_test_app();

        let (owner, _entry_node, _target, _graph) = {
            let world = app.world_mut();
            let owner = world
                .spawn((Observer::new(trigger_effect_graph_exec), EffectGraphOwner))
                .id();
            let entry_node = world.spawn_empty().id();
            let target = world.spawn_empty().id();
            let graph = world
                .spawn((
                    EffectGraphContext::new(),
                    EffectGraphExecutor::default(),
                    EffectGraphState::Inactive,
                ))
                .set_parent_in_place(owner)
                .id();
            {
                let mut context = world
                    .get_mut::<EffectGraphContext>(graph)
                    .expect("图上下文应存在");
                context.entry_node = Some(entry_node);
                context
                    .add_exec_connection(exec_pin(entry_node, "start"), &[exec_pin(target, "run")]);
            }
            (owner, entry_node, target, graph)
        };

        let received = Arc::new(Mutex::new(Vec::<EffectNodeExecPin>::new()));
        let received_obs = received.clone();
        app.add_observer(move |trigger: On<EffectNodeExecEvent>| {
            received_obs
                .lock()
                .expect("锁被占用")
                .push(trigger.event().input_exec_pin);
        });

        // 要求 Active 但图是 Inactive → 不执行。
        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(EffectGraphExecEvent {
                ability_entity: owner,
                entry_exec_pin: EffectNodeExec { name: "start" },
                execute_in_graph_state: Some(EffectGraphState::Active),
                slot_value_map: None,
            });
        });

        app.update();
        app.update();

        assert!(
            received.lock().expect("锁被占用").is_empty(),
            "图状态不匹配时不得执行"
        );
    }

    #[test]
    fn exec_writes_slot_value_map_to_entry_outputs() {
        let mut app = exec_test_app();

        let (owner, entry_node, graph) = {
            let world = app.world_mut();
            let owner = world
                .spawn((Observer::new(trigger_effect_graph_exec), EffectGraphOwner))
                .id();
            let entry_node = world.spawn_empty().id();
            let graph = world
                .spawn((
                    EffectGraphContext::new(),
                    EffectGraphExecutor::default(),
                    EffectGraphState::Inactive,
                ))
                .set_parent_in_place(owner)
                .id();
            {
                let mut context = world
                    .get_mut::<EffectGraphContext>(graph)
                    .expect("图上下文应存在");
                context.entry_node = Some(entry_node);
            }
            (owner, entry_node, graph)
        };

        let mut slot_value_map = HashMap::new();
        slot_value_map.insert(EffectNodeSlot::new::<f32>("damage"), EffectValue::F32(10.0));

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(EffectGraphExecEvent {
                ability_entity: owner,
                entry_exec_pin: EffectNodeExec { name: "start" },
                execute_in_graph_state: None,
                slot_value_map: Some(slot_value_map),
            });
        });

        app.update();
        app.update();

        let world = app.world();
        let context = world
            .entity(graph)
            .get::<EffectGraphContext>()
            .expect("图上下文应存在");
        assert_eq!(
            context.get_output_value(&slot_pin(entry_node, "damage")),
            Some(&EffectNodeSlotValue::Value(EffectValue::F32(10.0))),
            "slot_value_map 必须写入入口节点输出"
        );
    }

    #[test]
    fn exec_without_entry_node_does_nothing() {
        let mut app = exec_test_app();
        let (owner, _graph) = spawn_owner_with_graph(&mut app, EffectGraphState::Inactive, None);
        app.world_mut()
            .entity_mut(owner)
            .insert(Observer::new(trigger_effect_graph_exec));

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(EffectGraphExecEvent {
                ability_entity: owner,
                entry_exec_pin: EffectNodeExec { name: "start" },
                execute_in_graph_state: None,
                slot_value_map: None,
            });
        });

        app.update();
        app.update();
    }

    #[test]
    fn exec_ready_with_inactive_child_skips_clone() {
        let mut app = exec_test_app();
        let (owner, _graph) = spawn_owner_with_graph(&mut app, EffectGraphState::Inactive, None);
        app.world_mut()
            .entity_mut(owner)
            .insert(Observer::new(trigger_effect_graph_exec));

        let clone_received = Arc::new(Mutex::new(false));
        let clone_obs = clone_received.clone();
        app.add_observer(move |_trigger: On<CloneEffectGraphStartEvent>| {
            *clone_obs.lock().expect("锁被占用") = true;
        });

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(EffectGraphExecEvent {
                ability_entity: owner,
                entry_exec_pin: EffectNodeExec { name: "ready" },
                execute_in_graph_state: None,
                slot_value_map: None,
            });
        });

        app.update();
        app.update();

        assert!(
            !*clone_received.lock().expect("锁被占用"),
            "Inactive 图不得触发克隆"
        );
    }

    #[test]
    fn exec_ready_with_to_remove_child_falls_through() {
        let mut app = exec_test_app();
        let (owner, _graph) = spawn_owner_with_graph(&mut app, EffectGraphState::ToRemove, None);
        app.world_mut()
            .entity_mut(owner)
            .insert(Observer::new(trigger_effect_graph_exec));

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(EffectGraphExecEvent {
                ability_entity: owner,
                entry_exec_pin: EffectNodeExec { name: "ready" },
                execute_in_graph_state: None,
                slot_value_map: None,
            });
        });

        app.update();
        app.update();
    }

    // ===== trigger_effect_graph_add =====

    #[derive(Debug, Default, Reflect)]
    struct TestGraphBuilder;

    impl EffectGraphBuilder for TestGraphBuilder {
        fn get_effect_graph_name(&self) -> &'static str {
            "test_graph"
        }

        fn build(
            &self,
            commands: &mut Commands,
            _instant_map: &mut ResMut<InstantEffectNodeMap>,
        ) -> Entity {
            commands.spawn(EffectGraphOwner).id()
        }
    }

    #[test]
    fn add_event_clones_existing_graph_template() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<EffectGraphMap>()
            .init_resource::<EffectGraphBuilderMap>()
            .init_resource::<InstantEffectNodeMap>();
        app.add_observer(trigger_clone_effect_graph_start)
            .add_observer(trigger_clone_effect_graph_end);

        let (source, owner) = {
            let world = app.world_mut();
            let source = world.spawn_empty().id();
            let owner = world
                .spawn((Observer::new(trigger_effect_graph_add), EffectGraphOwner))
                .id();
            (source, owner)
        };
        app.world_mut()
            .resource_mut::<EffectGraphMap>()
            .insert_graph("existing".to_string(), GraphRef::new(source));

        let children_before = app
            .world()
            .entity(owner)
            .get::<Children>()
            .map(|c| c.len())
            .unwrap_or(0);

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(EffectGraphAddEvent {
                ability_entity: owner,
                graph_class: "existing".to_string(),
            });
        });

        app.update();
        app.update();
        app.update();

        let world = app.world();
        let children_after = world
            .entity(owner)
            .get::<Children>()
            .map(|c| c.len())
            .unwrap_or(0);
        assert_eq!(
            children_after,
            children_before + 1,
            "已存在图模板时必须克隆出新图实例"
        );
    }

    #[test]
    fn add_event_builds_graph_from_builder_then_clones() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<EffectGraphMap>()
            .init_resource::<EffectGraphBuilderMap>()
            .init_resource::<InstantEffectNodeMap>();
        app.add_observer(trigger_clone_effect_graph_start)
            .add_observer(trigger_clone_effect_graph_end);
        app.register_effect_graph_builder::<TestGraphBuilder>();

        let owner = app
            .world_mut()
            .spawn((Observer::new(trigger_effect_graph_add), EffectGraphOwner))
            .id();

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(EffectGraphAddEvent {
                ability_entity: owner,
                graph_class: "test_graph".to_string(),
            });
        });

        app.update();
        app.update();
        app.update();

        let world = app.world();
        let children = world
            .entity(owner)
            .get::<Children>()
            .expect("必须产生图实例子实体");
        assert!(!children.is_empty(), "builder 路径必须构建并克隆出新图实例");
    }

    #[test]
    fn add_event_without_builder_does_nothing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<EffectGraphMap>()
            .init_resource::<EffectGraphBuilderMap>()
            .init_resource::<InstantEffectNodeMap>();
        let owner = app
            .world_mut()
            .spawn((Observer::new(trigger_effect_graph_add), EffectGraphOwner))
            .id();

        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(EffectGraphAddEvent {
                ability_entity: owner,
                graph_class: "missing".to_string(),
            });
        });

        app.update();
        app.update();

        let world = app.world();
        assert!(
            world.entity(owner).get::<Children>().is_none(),
            "无模板无 builder 时不得产生子实体"
        );
    }

    // ===== trigger_clone_effect_graph_start / end =====

    #[test]
    fn clone_start_clones_source_into_destination() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.register_type::<Name>();
        app.add_observer(trigger_clone_effect_graph_start);

        let (source, dest) = {
            let world = app.world_mut();
            let source = world.spawn(Name::new("source_graph")).id();
            let dest = world
                .spawn(Observer::new(trigger_clone_effect_graph_end))
                .id();
            (source, dest)
        };

        let clone_event = CloneEffectGraphStartEvent {
            new_graph_instance: dest,
            graph_ref: GraphRef::new(source),
            destination_entity: dest,
        };
        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(clone_event);
        });

        app.update();
        app.update();

        let world = app.world();
        let name = world
            .entity(dest)
            .get::<Name>()
            .expect("克隆目标必须获得源实体组件");
        assert_eq!(name.as_str(), "source_graph");
        assert!(
            world.entity(dest).get::<EffectGraphExecutor>().is_some(),
            "clone_end 必须为新实例挂载执行器"
        );
    }

    #[test]
    fn clone_end_replaces_context_state_entities() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let (old_node, new_node, dest) = {
            let world = app.world_mut();
            let old_node = world.spawn_empty().id();
            let new_node = world.spawn_empty().id();
            let mut context = EffectGraphContext::new();
            context.entry_node = Some(old_node);
            let dest = world
                .spawn((context, Observer::new(trigger_clone_effect_graph_end)))
                .id();
            (old_node, new_node, dest)
        };

        let mut mapping = EntityHashMap::default();
        mapping.insert(old_node, new_node);
        let end_event = CloneEffectGraphEndEvent {
            destination_root_entity: dest,
            old_new_entities: mapping,
            new_graph_instance: dest,
        };
        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(end_event);
        });

        app.update();
        app.update();

        let world = app.world();
        let context = world
            .entity(dest)
            .get::<EffectGraphContext>()
            .expect("目标实体必须有上下文");
        assert_eq!(
            context.get_entry_node(),
            Some(new_node),
            "clone_end 必须重写旧实体引用"
        );
    }

    #[test]
    fn clone_end_without_context_logs_error() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let dest = app
            .world_mut()
            .spawn(Observer::new(trigger_clone_effect_graph_end))
            .id();
        let end_event = CloneEffectGraphEndEvent {
            destination_root_entity: dest,
            old_new_entities: EntityHashMap::default(),
            new_graph_instance: dest,
        };
        trigger_once(&mut app, move |mut commands: Commands| {
            commands.trigger(end_event);
        });

        app.update();
        app.update();

        let world = app.world();
        assert!(
            world.entity(dest).get::<EffectGraphExecutor>().is_some(),
            "无上下文时仍应挂载执行器"
        );
    }
}
