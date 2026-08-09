//! Effect Graph 执行器：按执行流连接驱动节点执行，直到状态节点为止。

use std::ops::Not;

use bevy::prelude::*;

use super::{
    EffectGraphUpdateSystems,
    context::{EffectGraphContext, InstantEffectNodeMap},
    event::EffectNodeExecEvent,
    node::{EffectNodeId, pin::EffectNodePinGroup},
    pin::EffectNodeExecPin,
};

/// 执行器插件：注册执行器系统（在 [`EffectGraphUpdateSystems::Execute`] 集内）。
///
/// 执行效果节点直到状态节点；状态节点也可能触发后续节点继续执行。
#[derive(Debug, Default)]
pub struct EffectGraphExecutorPlugin;

impl Plugin for EffectGraphExecutorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EffectGraphExecutor>().add_systems(
            Update,
            execute_graph.in_set(EffectGraphUpdateSystems::Execute),
        );
    }
}

/// 图的执行器组件：维护待执行的输出执行口队列，逐帧消费执行。
#[derive(Debug, Default, Component, Reflect)]
#[reflect(Component)]
pub struct EffectGraphExecutor {
    current_node_outputs: Vec<EffectNodeExecPin>,
}

impl EffectGraphExecutor {
    fn push_node_output_pin(&mut self, output_exec: EffectNodeExecPin) {
        self.current_node_outputs.push(output_exec);
    }

    // use to start execute next nodes of exec pin
    // push still to next node is state node.
    /// 从指定输出执行口启动执行：入队并立即沿连接推进后续节点。
    pub fn start_push_output_pin(
        &mut self,
        output_exec_pin: EffectNodeExecPin,
        context: &EffectGraphContext,
        instant_nodes: &Res<InstantEffectNodeMap>,
    ) {
        self.push_node_output_pin(output_exec_pin);
        self.continue_push_next_node_output_pin(output_exec_pin, context, instant_nodes);
    }

    // only use in push_execute_chain method  of instant node
    /// 沿输出执行口的连接推进后续节点（仅供即时节点的 `push_execute_chain` 使用）。
    pub fn continue_push_next_node_output_pin(
        &mut self,
        output_exec_pin: EffectNodeExecPin,
        context: &EffectGraphContext,
        instant_nodes: &Res<InstantEffectNodeMap>,
    ) {
        if let Some(next_input_exec_pins) = context.get_connected_output_exec_pins(&output_exec_pin)
        {
            for next_input_exec_pin in next_input_exec_pins {
                if let EffectNodeId::Uuid(uuid) = next_input_exec_pin.node_id
                    && let Some(node) = instant_nodes.get(uuid)
                {
                    node.push_execute_chain(context, self, next_input_exec_pin.exec, instant_nodes);
                }
            }
        }
    }

    // only use in push_execute_chain method  of instant node
    /// 按输出执行口名称推进后续节点（仅供即时节点的 `push_execute_chain` 使用）。
    pub fn continue_push_next_node_output_pin_from_node_name(
        &mut self,
        node_id: EffectNodeId,
        node: &impl EffectNodePinGroup,
        output_exec_pin_name: &str,
        context: &EffectGraphContext,
        instant_nodes: &Res<InstantEffectNodeMap>,
    ) {
        if let Some(pin) = node.get_output_exec_pin_by_name(output_exec_pin_name) {
            self.continue_push_next_node_output_pin(
                EffectNodeExecPin {
                    node_id,
                    exec: *pin,
                },
                context,
                instant_nodes,
            );
        }
    }
}

fn execute_graph(
    mut commands: Commands,
    mut query: Query<(&mut EffectGraphContext, &mut EffectGraphExecutor)>,
    instant_nodes: Res<InstantEffectNodeMap>,
) {
    for (mut context, mut executor) in query.iter_mut() {
        while executor.current_node_outputs.is_empty().not() {
            // info!(
            //     "executor.current_node_outputs: {:?}",
            //     executor.current_node_outputs
            // );
            // dbg!(&context);
            let current = executor.current_node_outputs.remove(0);
            if let Some(next_input_exec_pins) =
                context.get_connected_output_exec_pins(&current).cloned()
            {
                info!("next_input_exec_pins: {:?}", next_input_exec_pins);
                for next_input_exec_pin in next_input_exec_pins {
                    match next_input_exec_pin.node_id {
                        EffectNodeId::Uuid(uuid) => {
                            if let Some(node) = instant_nodes.get(uuid) {
                                node.execute(&mut context);
                            }
                        }
                        EffectNodeId::Entity(entity) => {
                            assert_ne!(entity, Entity::PLACEHOLDER);
                            let event = EffectNodeExecEvent {
                                input_exec_pin: next_input_exec_pin,
                            };
                            commands.trigger(event);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::pin::EffectNodeExec;
    use crate::graph::node::{InstantEffectNode, implement::seq::EffectNodeSeq};
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    /// 记录型即时节点：记录 push_execute_chain 的输入执行口名与 execute 调用次数。
    struct RecordingNode {
        uuid: Uuid,
        pushed: Arc<Mutex<Vec<&'static str>>>,
        executed: Arc<Mutex<usize>>,
    }

    impl InstantEffectNode for RecordingNode {
        fn get_uuid(&self) -> Uuid {
            self.uuid
        }

        fn push_execute_chain(
            &self,
            _context: &EffectGraphContext,
            _executor: &mut EffectGraphExecutor,
            input_exec_pin: EffectNodeExec,
            _instant_nodes: &Res<InstantEffectNodeMap>,
        ) {
            self.pushed
                .lock()
                .expect("锁被占用")
                .push(input_exec_pin.name);
        }

        fn collect(&self, _context: &mut EffectGraphContext) {}

        fn execute(&self, _context: &mut EffectGraphContext) {
            *self.executed.lock().expect("锁被占用") += 1;
        }
    }

    fn exec_pin(node_id: EffectNodeId, name: &'static str) -> EffectNodeExecPin {
        EffectNodeExecPin {
            node_id,
            exec: EffectNodeExec { name },
        }
    }

    /// 注册记录节点到即时节点表，返回观测句柄。
    fn register_recording_node(
        app: &mut App,
        uuid: Uuid,
    ) -> (Arc<Mutex<Vec<&'static str>>>, Arc<Mutex<usize>>) {
        let pushed = Arc::new(Mutex::new(Vec::<&'static str>::new()));
        let executed = Arc::new(Mutex::new(0usize));
        let node: Arc<dyn InstantEffectNode> = Arc::new(RecordingNode {
            uuid,
            pushed: pushed.clone(),
            executed: executed.clone(),
        });
        app.world_mut()
            .resource_mut::<InstantEffectNodeMap>()
            .insert(uuid, node);
        (pushed, executed)
    }

    /// 以 seed 系统调用 executor 方法，确保在 execute_graph 之前运行。
    fn seed_start_push(app: &mut App, graph: Entity, pin: EffectNodeExecPin) {
        app.add_systems(
            Update,
            (move |mut q: Query<(&mut EffectGraphExecutor, &EffectGraphContext)>,
                   instant: Res<InstantEffectNodeMap>| {
                let (mut executor, context) = q.get_mut(graph).expect("图实体必须有执行器与上下文");
                executor.start_push_output_pin(pin, context, &instant);
            })
            .before(execute_graph),
        );
    }

    #[test]
    fn start_push_output_pin_without_connection_just_queues() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, EffectGraphExecutorPlugin));
        app.insert_resource(InstantEffectNodeMap::default());

        let graph = app
            .world_mut()
            .spawn((EffectGraphContext::new(), EffectGraphExecutor::default()))
            .id();
        seed_start_push(
            &mut app,
            graph,
            exec_pin(Entity::from_bits(1).into(), "start"),
        );

        app.update();

        // 无连接：execute_graph 消费空队列后不产生事件，执行器保持存在。
        assert!(
            app.world().get_entity(graph).is_ok(),
            "无连接时图实体不受影响"
        );
    }

    #[test]
    fn execute_graph_triggers_entity_node_event() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, EffectGraphExecutorPlugin));
        app.insert_resource(InstantEffectNodeMap::default());

        let world = app.world_mut();
        let node_a = world.spawn_empty().id();
        let node_b = world.spawn_empty().id();
        let mut context = EffectGraphContext::new();
        context.add_exec_connection(
            exec_pin(node_a.into(), "start"),
            &[exec_pin(node_b.into(), "run")],
        );
        let graph = world.spawn((context, EffectGraphExecutor::default())).id();
        seed_start_push(&mut app, graph, exec_pin(node_a.into(), "start"));

        let received = Arc::new(Mutex::new(Vec::<EffectNodeExecPin>::new()));
        let received_obs = received.clone();
        app.add_observer(move |trigger: On<EffectNodeExecEvent>| {
            received_obs
                .lock()
                .expect("锁被占用")
                .push(trigger.event().input_exec_pin);
        });

        app.update();

        let events = received.lock().expect("锁被占用");
        assert_eq!(
            events.as_slice(),
            &[exec_pin(node_b.into(), "run")],
            "execute_graph 必须为实体节点触发 EffectNodeExecEvent"
        );
    }

    #[test]
    fn continue_push_next_node_output_pin_calls_instant_node() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, EffectGraphExecutorPlugin));
        app.insert_resource(InstantEffectNodeMap::default());

        let source_uuid = Uuid::new_v4();
        let (pushed, executed) = register_recording_node(&mut app, source_uuid);

        let world = app.world_mut();
        let mut context = EffectGraphContext::new();
        context.add_exec_connection(
            exec_pin(Entity::from_bits(1).into(), "start"),
            &[exec_pin(source_uuid.into(), "trigger")],
        );
        let graph = world.spawn((context, EffectGraphExecutor::default())).id();
        seed_start_push(
            &mut app,
            graph,
            exec_pin(Entity::from_bits(1).into(), "start"),
        );

        app.update();

        assert_eq!(
            pushed.lock().expect("锁被占用").as_slice(),
            &["trigger"],
            "即时节点的 push_execute_chain 必须收到后续执行口"
        );
        assert_eq!(
            *executed.lock().expect("锁被占用"),
            1,
            "execute_graph 必须执行即时节点"
        );
    }

    #[test]
    fn continue_push_next_node_output_pin_from_node_name_delegates() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(InstantEffectNodeMap::default());

        let seq_uuid = Uuid::new_v4();
        let (pushed, _executed) = register_recording_node(&mut app, seq_uuid);

        let world = app.world_mut();
        let mut context = EffectGraphContext::new();
        context.add_exec_connection(
            exec_pin(seq_uuid.into(), "finish_1"),
            &[exec_pin(seq_uuid.into(), "next")],
        );
        let graph = world.spawn((context, EffectGraphExecutor::default())).id();

        let seq = EffectNodeSeq::new();
        let pin = exec_pin(seq_uuid.into(), "finish_1");
        let seq_id = pin.node_id;
        app.add_systems(
            Update,
            (move |mut q: Query<&mut EffectGraphExecutor>,
                   ctx_q: Query<&EffectGraphContext>,
                   instant: Res<InstantEffectNodeMap>| {
                let mut executor = q.get_mut(graph).expect("图实体必须有执行器");
                let context = ctx_q.get(graph).expect("图实体必须有上下文");
                executor.continue_push_next_node_output_pin_from_node_name(
                    seq_id, &seq, "finish_1", context, &instant,
                );
            })
            .before(execute_graph),
        );

        app.update();

        assert_eq!(
            pushed.lock().expect("锁被占用").as_slice(),
            &["next"],
            "按名称推进必须沿 finish_1 连接触发目标即时节点"
        );
    }

    #[test]
    fn continue_push_from_node_name_with_missing_name_is_noop() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(InstantEffectNodeMap::default());

        let seq_uuid = Uuid::new_v4();
        let (pushed, _executed) = register_recording_node(&mut app, seq_uuid);

        let world = app.world_mut();
        let context = EffectGraphContext::new();
        let graph = world.spawn((context, EffectGraphExecutor::default())).id();

        let seq = EffectNodeSeq::new();
        app.add_systems(
            Update,
            (move |mut q: Query<&mut EffectGraphExecutor>,
                   ctx_q: Query<&EffectGraphContext>,
                   instant: Res<InstantEffectNodeMap>| {
                let mut executor = q.get_mut(graph).expect("图实体必须有执行器");
                let context = ctx_q.get(graph).expect("图实体必须有上下文");
                executor.continue_push_next_node_output_pin_from_node_name(
                    seq_uuid.into(),
                    &seq,
                    "nonexistent",
                    context,
                    &instant,
                );
            })
            .before(execute_graph),
        );

        app.update();

        assert!(
            pushed.lock().expect("锁被占用").is_empty(),
            "不存在的输出执行口名不得触发任何推进"
        );
    }

    #[test]
    #[should_panic(expected = "left != right")]
    fn execute_graph_asserts_entity_is_not_placeholder() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, EffectGraphExecutorPlugin));
        app.insert_resource(InstantEffectNodeMap::default());

        let world = app.world_mut();
        let node_a = world.spawn_empty().id();
        let mut context = EffectGraphContext::new();
        context.add_exec_connection(
            exec_pin(node_a.into(), "start"),
            &[exec_pin(Entity::PLACEHOLDER.into(), "run")],
        );
        let graph = world.spawn((context, EffectGraphExecutor::default())).id();
        seed_start_push(&mut app, graph, exec_pin(node_a.into(), "start"));

        app.update();
    }
}
