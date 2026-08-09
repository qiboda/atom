//! 顺序分支节点：同时触发 finish_1 ~ finish_4 四条输出分支。

use std::vec::Vec;

use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    graph::{
        context::{EffectGraphContext, InstantEffectNodeMap},
        executor::EffectGraphExecutor,
        node::{EffectNode, InstantEffectNode, InstantEffectNodeBase, pin::EffectNodeExec},
    },
    impl_effect_node_pin_group,
};

/// 顺序分支节点插件：注册 [`EffectNodeSeq`] 类型反射。
#[derive(Debug, Default)]
pub struct EffectNodeSeqPlugin;

impl Plugin for EffectNodeSeqPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EffectNodeSeq>();
    }
}

///////////////////////// Node Component /////////////////////////

/// 顺序分支即时节点：start 后同时沿 finish_1 ~ finish_4 继续执行。
#[derive(Debug, Default, Reflect)]
pub struct EffectNodeSeq {
    /// 即时节点基础结构（UUID）。
    pub base: InstantEffectNodeBase,
}

impl EffectNodeSeq {
    /// 创建带新 UUID 的顺序分支节点。
    pub fn new() -> Self {
        Self {
            base: InstantEffectNodeBase::new(),
        }
    }
}

impl_effect_node_pin_group!(EffectNodeSeq,
    input => (
        start => ()
    )
    output => (
        finish_1 => (),
        finish_2 => (),
        finish_3 => (),
        finish_4 => ()
    )
);

impl EffectNode for EffectNodeSeq {}

impl InstantEffectNode for EffectNodeSeq {
    fn get_uuid(&self) -> Uuid {
        self.base.node_id
    }

    fn collect(&self, _context: &mut EffectGraphContext) {}

    fn push_execute_chain(
        &self,
        context: &EffectGraphContext,
        executor: &mut EffectGraphExecutor,
        _input_exec_pin: EffectNodeExec,
        instant_nodes: &Res<InstantEffectNodeMap>,
    ) {
        executor.continue_push_next_node_output_pin_from_node_name(
            self.get_uuid().into(),
            self,
            EffectNodeSeq::OUTPUT_EXEC_FINISH_1,
            context,
            instant_nodes,
        );
        executor.continue_push_next_node_output_pin_from_node_name(
            self.get_uuid().into(),
            self,
            EffectNodeSeq::OUTPUT_EXEC_FINISH_2,
            context,
            instant_nodes,
        );
        executor.continue_push_next_node_output_pin_from_node_name(
            self.get_uuid().into(),
            self,
            EffectNodeSeq::OUTPUT_EXEC_FINISH_3,
            context,
            instant_nodes,
        );
        executor.continue_push_next_node_output_pin_from_node_name(
            self.get_uuid().into(),
            self,
            EffectNodeSeq::OUTPUT_EXEC_FINISH_4,
            context,
            instant_nodes,
        );
    }

    fn execute(&self, _context: &mut EffectGraphContext) {
        info!(
            "node {} execute: {:?}",
            std::any::type_name::<EffectNodeSeq>(),
            self.get_uuid(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::node::pin::EffectNodePinGroup;
    use crate::graph::pin::EffectNodeExecPin;

    #[test]
    fn new_creates_node_with_non_nil_uuid() {
        let node = EffectNodeSeq::new();
        assert!(!node.get_uuid().is_nil());
        assert_eq!(node.get_uuid(), node.base.node_id);
    }

    #[test]
    fn default_creates_nil_uuid() {
        let node = EffectNodeSeq::default();
        assert!(node.get_uuid().is_nil());
    }

    #[test]
    fn new_nodes_have_distinct_uuids() {
        let node_a = EffectNodeSeq::new();
        let node_b = EffectNodeSeq::new();
        assert_ne!(node_a.get_uuid(), node_b.get_uuid());
    }

    #[test]
    fn pin_groups_expose_start_input_and_four_finish_outputs() {
        let node = EffectNodeSeq::new();

        assert_eq!(node.get_input_pin_group_num(), 1);
        assert_eq!(node.get_output_pin_group_num(), 4);

        assert_eq!(
            node.get_input_exec_pin_by_name(EffectNodeSeq::INPUT_EXEC_START)
                .map(|exec| exec.name),
            Some(EffectNodeSeq::INPUT_EXEC_START)
        );
        for name in [
            EffectNodeSeq::OUTPUT_EXEC_FINISH_1,
            EffectNodeSeq::OUTPUT_EXEC_FINISH_2,
            EffectNodeSeq::OUTPUT_EXEC_FINISH_3,
            EffectNodeSeq::OUTPUT_EXEC_FINISH_4,
        ] {
            assert_eq!(
                node.get_output_exec_pin_by_name(name).map(|exec| exec.name),
                Some(name),
            );
        }
    }

    #[test]
    fn pin_group_queries_missing_names_return_none() {
        let node = EffectNodeSeq::new();
        assert_eq!(node.get_input_exec_pin_by_name("nonexistent"), None);
        assert_eq!(node.get_output_exec_pin_by_name("nonexistent"), None);
        assert!(node.get_input_pin_group_by_name("nonexistent").is_none());
        assert!(node.get_output_pin_group_by_name("nonexistent").is_none());
    }

    #[test]
    fn start_input_group_has_no_slots() {
        let node = EffectNodeSeq::new();
        let group = node
            .get_input_pin_group_by_name(EffectNodeSeq::INPUT_EXEC_START)
            .expect("start 输入组应存在");
        assert!(group.slots.is_empty());
        assert_eq!(node.get_input_slot_pin_by_name("anything"), None);
    }

    #[test]
    fn collect_leaves_context_untouched() {
        let node = EffectNodeSeq::new();
        let mut context = EffectGraphContext::new();
        context.insert_state_node(Entity::from_bits(1));

        node.collect(&mut context);

        assert_eq!(context.state_nodes, vec![Entity::from_bits(1)]);
        assert!(context.inputs.is_empty());
        assert!(context.outputs.is_empty());
    }

    #[test]
    fn execute_is_noop_on_empty_context() {
        let node = EffectNodeSeq::new();
        let mut context = EffectGraphContext::new();
        node.execute(&mut context);
        assert!(context.inputs.is_empty());
        assert!(context.outputs.is_empty());
    }

    #[test]
    fn plugin_registers_seq_type() {
        let mut app = App::new();
        app.add_plugins(EffectNodeSeqPlugin);
        app.update();
    }

    #[test]
    fn push_execute_chain_triggers_all_four_finish_outputs() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(InstantEffectNodeMap::default());

        let seq = EffectNodeSeq::new();
        let seq_uuid = seq.get_uuid();

        // 四个 finish 输出各连接一个记录节点。
        let target_uuids = [
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
            Uuid::new_v4(),
        ];
        let target_names = ["t1", "t2", "t3", "t4"];
        let mut pushed_handles = Vec::new();
        for uuid in target_uuids.iter() {
            let pushed = std::sync::Arc::new(std::sync::Mutex::new(Vec::<&'static str>::new()));
            let pushed_node = pushed.clone();
            let node: std::sync::Arc<dyn InstantEffectNode> = std::sync::Arc::new(TargetNode {
                uuid: *uuid,
                pushed: pushed_node,
            });
            app.world_mut()
                .resource_mut::<InstantEffectNodeMap>()
                .insert(*uuid, node);
            pushed_handles.push(pushed);
        }

        let world = app.world_mut();
        let mut context = EffectGraphContext::new();
        for (i, finish) in [
            EffectNodeSeq::OUTPUT_EXEC_FINISH_1,
            EffectNodeSeq::OUTPUT_EXEC_FINISH_2,
            EffectNodeSeq::OUTPUT_EXEC_FINISH_3,
            EffectNodeSeq::OUTPUT_EXEC_FINISH_4,
        ]
        .into_iter()
        .enumerate()
        {
            context.add_exec_connection(
                EffectNodeExecPin {
                    node_id: seq_uuid.into(),
                    exec: EffectNodeExec { name: finish },
                },
                &[EffectNodeExecPin {
                    node_id: target_uuids[i].into(),
                    exec: EffectNodeExec {
                        name: target_names[i],
                    },
                }],
            );
        }
        let graph = world.spawn((context, EffectGraphExecutor::default())).id();

        app.add_systems(
            Update,
            (
                move |mut q: Query<&mut EffectGraphExecutor>,
                      ctx_q: Query<&EffectGraphContext>,
                      instant: Res<InstantEffectNodeMap>| {
                    let mut executor = q.get_mut(graph).expect("图实体必须有执行器");
                    let context = ctx_q.get(graph).expect("图实体必须有上下文");
                    seq.push_execute_chain(
                        context,
                        &mut executor,
                        EffectNodeExec { name: "start" },
                        &instant,
                    );
                },
            ),
        );

        app.update();

        for (i, handle) in pushed_handles.iter().enumerate() {
            let pushed = handle.lock().expect("锁被占用");
            assert_eq!(
                pushed.as_slice(),
                &[target_names[i]],
                "finish_{} 必须触发目标节点",
                i + 1
            );
        }
    }

    /// 记录 push_execute_chain 输入名的测试节点。
    struct TargetNode {
        uuid: Uuid,
        pushed: std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
    }

    impl InstantEffectNode for TargetNode {
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

        fn execute(&self, _context: &mut EffectGraphContext) {}
    }
}
