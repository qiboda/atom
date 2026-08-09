//! 计时状态节点：start 时按 duration 启动计时，到时触发 finish。

use bevy::{prelude::*, time::Time};

use crate::{
    graph::{
        EffectGraphUpdateSystems,
        context::{EffectGraphContext, InstantEffectNodeMap},
        event::EffectNodeExecEvent,
        executor::EffectGraphExecutor,
        node::{
            EffectNode, EffectNodeExecuteState, EffectNodeId, StateEffectNode, pin::EffectNodeExec,
        },
        pin::EffectNodeExecPin,
        state::EffectGraphTickState,
    },
    impl_effect_node_pin_group,
};

/// 计时节点插件：注册类型反射、节点事件 observer 与计时更新系统。
#[derive(Debug)]
pub struct EffectNodeTimerPlugin;

impl Plugin for EffectNodeTimerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EffectNodeTimer>()
            .add_observer(trigger_effect_node_event)
            .add_systems(
                Update,
                update_timer.in_set(EffectGraphUpdateSystems::UpdateNode),
            );
    }
}

/// 单个计时实例的剩余时间。
#[derive(Clone, Debug, Default, Reflect)]
pub struct EffectNodeTimerState {
    /// 剩余时间（秒），递减到 0 触发 finish。
    pub elapse: f32,
}

/// 计时状态节点：维护多个并发计时实例。
#[derive(Clone, Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct EffectNodeTimer {
    /// 正在进行的计时实例列表。
    pub states: Vec<EffectNodeTimerState>,
}

impl EffectNode for EffectNodeTimer {}

impl StateEffectNode for EffectNodeTimer {}

impl_effect_node_pin_group!(EffectNodeTimer,
    input => (
        start => (duration: f32)
    )
    output => (
        start => (),
        finish => ()
    )
);

fn trigger_effect_node_event(
    trigger: On<EffectNodeExecEvent>,
    mut query: Query<(&mut EffectNodeTimer, &mut EffectNodeExecuteState, &ChildOf)>,
    mut graph_query: Query<(&EffectGraphContext, &mut EffectGraphExecutor)>,
    instant_nodes: Res<InstantEffectNodeMap>,
) {
    let pin = trigger.event().input_exec_pin;
    let EffectNodeId::Entity(entity) = pin.node_id else {
        return;
    };

    if let Ok((mut node, mut state, parent)) = query.get_mut(entity) {
        info!("trigger_node_event: timer {:?}", pin);

        if let Ok((context, mut executor)) = graph_query.get_mut(parent.parent())
            && let EffectNodeTimer::INPUT_EXEC_START = pin.exec.name
        {
            let duration_value = context.get_input_value_type_from_node::<&f32>(
                entity,
                &*node,
                EffectNodeTimer::INPUT_SLOT_DURATION,
            );

            if let Some(duration) = duration_value {
                node.states.push(EffectNodeTimerState { elapse: *duration });
            }

            if *state == EffectNodeExecuteState::Idle {
                *state = EffectNodeExecuteState::Active;
            }

            executor.start_push_output_pin(
                EffectNodeExecPin {
                    node_id: entity.into(),
                    exec: EffectNodeTimer::OUTPUT_EXEC_START.into(),
                },
                context,
                &instant_nodes,
            );
        }
    }
}

fn update_timer(
    mut graph_query: Query<(
        &EffectGraphContext,
        &mut EffectGraphExecutor,
        &EffectGraphTickState,
    )>,
    mut query: Query<(
        Entity,
        &mut EffectNodeTimer,
        &mut EffectNodeExecuteState,
        &ChildOf,
    )>,
    instant_map: Res<InstantEffectNodeMap>,
    time: Res<Time>,
) {
    for (entity, mut node, mut node_state, parent) in query.iter_mut() {
        if *node_state == EffectNodeExecuteState::Idle {
            continue;
        }

        match graph_query.get_mut(parent.parent()) {
            Ok((context, mut executor, tick_state)) => {
                if *tick_state != EffectGraphTickState::Ticked {
                    continue;
                }
                for state in node.states.iter_mut() {
                    state.elapse -= time.delta_secs();
                    if state.elapse <= 0.0 {
                        executor.start_push_output_pin(
                            EffectNodeExecPin {
                                node_id: EffectNodeId::Entity(entity),
                                exec: EffectNodeExec {
                                    name: EffectNodeTimer::OUTPUT_EXEC_FINISH,
                                },
                            },
                            context,
                            &instant_map,
                        );
                    }
                }
            }
            Err(e) => {
                error!("update_timer error: {}", e);
            }
        }

        node.states.retain(|state| state.elapse > 0.0);

        if node.states.is_empty() {
            *node_state = EffectNodeExecuteState::Idle;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::EffectGraphPlugin;
    use crate::graph::blackboard::EffectValue;
    use crate::graph::event::EffectNodeExecEvent;
    use crate::graph::node::pin::{EffectNodeExec, EffectNodePinGroup, EffectNodeSlot};
    use crate::graph::pin::{EffectNodeExecPin, EffectNodeSlotPin};
    use bevy::time::TimeUpdateStrategy;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use uuid::Uuid;

    fn exec_pin(node_id: EffectNodeId, name: &'static str) -> EffectNodeExecPin {
        EffectNodeExecPin {
            node_id,
            exec: EffectNodeExec { name },
        }
    }

    fn spawn_graph_and_node(app: &mut App, graph_tick: EffectGraphTickState) -> (Entity, Entity) {
        let world = app.world_mut();
        let graph = world
            .spawn((
                EffectGraphContext::new(),
                EffectGraphExecutor::default(),
                graph_tick,
            ))
            .id();
        let node = world
            .spawn((EffectNodeTimer::default(), EffectNodeExecuteState::Idle))
            .set_parent_in_place(graph)
            .id();
        (graph, node)
    }

    #[test]
    fn timer_struct_defaults() {
        let state = EffectNodeTimerState::default();
        assert_eq!(state.elapse, 0.0);

        let node = EffectNodeTimer::default();
        assert!(node.states.is_empty());
    }

    #[test]
    fn timer_pin_groups_expose_duration_slot_and_start_finish_outputs() {
        let node = EffectNodeTimer::default();
        assert_eq!(node.get_input_pin_group_num(), 1);
        assert_eq!(node.get_output_pin_group_num(), 2);

        let duration = node
            .get_input_slot_pin_by_name(EffectNodeTimer::INPUT_SLOT_DURATION)
            .expect("duration 输入槽应存在");
        assert_eq!(duration.pin_type, std::any::TypeId::of::<f32>());

        assert_eq!(
            node.get_output_exec_pin_by_name(EffectNodeTimer::OUTPUT_EXEC_START)
                .map(|exec| exec.name),
            Some(EffectNodeTimer::OUTPUT_EXEC_START)
        );
        assert_eq!(
            node.get_output_exec_pin_by_name(EffectNodeTimer::OUTPUT_EXEC_FINISH)
                .map(|exec| exec.name),
            Some(EffectNodeTimer::OUTPUT_EXEC_FINISH)
        );
    }

    #[test]
    fn plugin_builds_without_panicking() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, EffectNodeTimerPlugin));
        app.init_resource::<InstantEffectNodeMap>();
        app.update();
    }

    #[test]
    fn trigger_start_event_pushes_duration_and_activates_node() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, EffectGraphPlugin, EffectNodeTimerPlugin));

        let (graph, node) = spawn_graph_and_node(&mut app, EffectGraphTickState::Ticked);
        {
            let world = app.world_mut();
            let mut context = world
                .get_mut::<EffectGraphContext>(graph)
                .expect("图实体必须有上下文");
            context.insert_input_value(
                EffectNodeSlotPin {
                    node_id: node.into(),
                    slot: EffectNodeSlot::new::<f32>(EffectNodeTimer::INPUT_SLOT_DURATION),
                },
                EffectValue::F32(3.0).into(),
            );
        }

        app.add_systems(Update, move |mut commands: Commands| {
            commands.trigger(EffectNodeExecEvent {
                input_exec_pin: exec_pin(node.into(), EffectNodeTimer::INPUT_EXEC_START),
            });
        });

        app.update();

        let world = app.world();
        let timer = world
            .entity(node)
            .get::<EffectNodeTimer>()
            .expect("节点必须有计时组件");
        assert_eq!(timer.states.len(), 1, "start 触发后必须记录一个计时实例");
        assert_eq!(
            timer.states[0].elapse, 3.0,
            "计时实例必须读取 duration 输入"
        );
        let state = world
            .entity(node)
            .get::<EffectNodeExecuteState>()
            .expect("节点必须有执行状态");
        assert_eq!(*state, EffectNodeExecuteState::Active);
    }

    #[test]
    fn trigger_start_event_without_duration_still_activates() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, EffectGraphPlugin, EffectNodeTimerPlugin));

        let (_graph, node) = spawn_graph_and_node(&mut app, EffectGraphTickState::Ticked);

        app.add_systems(Update, move |mut commands: Commands| {
            commands.trigger(EffectNodeExecEvent {
                input_exec_pin: exec_pin(node.into(), EffectNodeTimer::INPUT_EXEC_START),
            });
        });

        app.update();

        let world = app.world();
        let timer = world
            .entity(node)
            .get::<EffectNodeTimer>()
            .expect("节点必须有计时组件");
        assert!(timer.states.is_empty(), "无 duration 输入时不创建计时实例");
        let state = world
            .entity(node)
            .get::<EffectNodeExecuteState>()
            .expect("节点必须有执行状态");
        // observer 置 Active 后，update_timer 发现无计时实例又将其置回 Idle。
        assert_eq!(*state, EffectNodeExecuteState::Idle);
    }

    #[test]
    fn trigger_event_with_uuid_pin_is_ignored() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, EffectGraphPlugin, EffectNodeTimerPlugin));

        app.add_systems(Update, |mut commands: Commands| {
            commands.trigger(EffectNodeExecEvent {
                input_exec_pin: exec_pin(Uuid::nil().into(), "start"),
            });
        });

        app.update();
    }

    #[test]
    fn trigger_event_for_node_without_parent_is_ignored() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, EffectGraphPlugin, EffectNodeTimerPlugin));
        let node = app
            .world_mut()
            .spawn((EffectNodeTimer::default(), EffectNodeExecuteState::Idle))
            .id();

        app.add_systems(Update, move |mut commands: Commands| {
            commands.trigger(EffectNodeExecEvent {
                input_exec_pin: exec_pin(node.into(), EffectNodeTimer::INPUT_EXEC_START),
            });
        });

        app.update();

        let timer = app
            .world()
            .entity(node)
            .get::<EffectNodeTimer>()
            .expect("节点必须有计时组件");
        assert!(timer.states.is_empty());
    }

    #[test]
    fn update_timer_decrements_then_fires_finish_and_returns_idle() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, EffectGraphPlugin, EffectNodeTimerPlugin));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            0.5,
        )));

        let (graph, node) = spawn_graph_and_node(&mut app, EffectGraphTickState::Ticked);
        {
            let world = app.world_mut();
            let mut timer = world
                .get_mut::<EffectNodeTimer>(node)
                .expect("节点必须有计时组件");
            timer.states = vec![EffectNodeTimerState { elapse: 1.0 }];
            *world
                .get_mut::<EffectNodeExecuteState>(node)
                .expect("节点必须有执行状态") = EffectNodeExecuteState::Active;
            let next = world.spawn_empty().id();
            let mut context = world
                .get_mut::<EffectGraphContext>(graph)
                .expect("图实体必须有上下文");
            context.add_exec_connection(
                exec_pin(node.into(), EffectNodeTimer::OUTPUT_EXEC_FINISH),
                &[exec_pin(next.into(), "run")],
            );
            app.world_mut()
                .entity_mut(node)
                .insert(TestFinishMarker { next });
        }

        let received = Arc::new(Mutex::new(Vec::<EffectNodeExecPin>::new()));
        let received_obs = received.clone();
        app.add_observer(move |trigger: On<EffectNodeExecEvent>| {
            received_obs
                .lock()
                .expect("锁被占用")
                .push(trigger.event().input_exec_pin);
        });

        app.update();
        app.update();
        app.update();
        app.update();
        app.update();
        app.update();

        let world = app.world();
        let timer = world
            .entity(node)
            .get::<EffectNodeTimer>()
            .expect("节点必须有计时组件");
        assert!(timer.states.is_empty(), "计时归零后实例必须被清除");
        let state = world
            .entity(node)
            .get::<EffectNodeExecuteState>()
            .expect("节点必须有执行状态");
        assert_eq!(
            *state,
            EffectNodeExecuteState::Idle,
            "计时清空后节点回到 Idle"
        );

        let next = world
            .entity(node)
            .get::<TestFinishMarker>()
            .expect("标记应存在")
            .next;
        let events = received.lock().expect("锁被占用");
        assert!(
            events.contains(&exec_pin(next.into(), "run")),
            "finish 输出必须沿连接触发目标节点"
        );
    }

    #[derive(Component)]
    struct TestFinishMarker {
        next: Entity,
    }

    #[test]
    fn update_timer_skips_idle_nodes() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, EffectGraphPlugin, EffectNodeTimerPlugin));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            0.5,
        )));

        let (_graph, node) = spawn_graph_and_node(&mut app, EffectGraphTickState::Ticked);
        {
            let world = app.world_mut();
            let mut timer = world
                .get_mut::<EffectNodeTimer>(node)
                .expect("节点必须有计时组件");
            timer.states = vec![EffectNodeTimerState { elapse: 1.0 }];
        }

        app.update();

        let world = app.world();
        let timer = world
            .entity(node)
            .get::<EffectNodeTimer>()
            .expect("节点必须有计时组件");
        assert_eq!(timer.states[0].elapse, 1.0, "Idle 节点不得递减计时");
    }

    #[test]
    fn update_timer_skips_paused_graphs() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, EffectGraphPlugin, EffectNodeTimerPlugin));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            0.5,
        )));

        let (_graph, node) = spawn_graph_and_node(&mut app, EffectGraphTickState::Paused);
        {
            let world = app.world_mut();
            let mut timer = world
                .get_mut::<EffectNodeTimer>(node)
                .expect("节点必须有计时组件");
            timer.states = vec![EffectNodeTimerState { elapse: 1.0 }];
            *world
                .get_mut::<EffectNodeExecuteState>(node)
                .expect("节点必须有执行状态") = EffectNodeExecuteState::Active;
        }

        app.update();

        let world = app.world();
        let timer = world
            .entity(node)
            .get::<EffectNodeTimer>()
            .expect("节点必须有计时组件");
        assert_eq!(timer.states[0].elapse, 1.0, "暂停图不得递减计时");
    }

    #[test]
    fn update_timer_logs_error_when_parent_graph_missing() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, EffectGraphPlugin, EffectNodeTimerPlugin));
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            0.5,
        )));

        // 节点有父实体但父实体无图组件 → error 分支。
        let parent = app.world_mut().spawn_empty().id();
        let node = app
            .world_mut()
            .spawn((
                EffectNodeTimer {
                    states: vec![EffectNodeTimerState { elapse: 5.0 }],
                },
                EffectNodeExecuteState::Active,
            ))
            .set_parent_in_place(parent)
            .id();

        app.update();

        let timer = app
            .world()
            .entity(node)
            .get::<EffectNodeTimer>()
            .expect("节点必须有计时组件");
        assert_eq!(timer.states[0].elapse, 5.0, "图缺失时不递减计时");
    }
}
