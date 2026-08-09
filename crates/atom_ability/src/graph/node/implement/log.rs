//! 日志节点：执行时打印输入消息并沿 finish 执行口继续。

use std::vec::Vec;

use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    graph::{
        context::EffectGraphContext,
        node::{
            EffectNode, EffectNodeId, InstantEffectNode, InstantEffectNodeBase,
            pin::{EffectNodeExec, EffectNodePinGroup},
        },
        pin::EffectNodeSlotPin,
    },
    impl_effect_node_pin_group,
};

/// 日志节点插件：注册 [`EffectNodeLog`] 类型反射。
#[derive(Debug, Default)]
pub struct EffectNodeLogPlugin;

impl Plugin for EffectNodeLogPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<EffectNodeLog>();
    }
}

///////////////////////// Node Component /////////////////////////

/// 日志即时节点：输入 `message` 槽，执行时打印消息，随后触发 finish 执行口。
#[derive(Debug, Default, Reflect)]
pub struct EffectNodeLog {
    /// 即时节点基础结构（UUID）。
    pub base: InstantEffectNodeBase,
}

impl EffectNodeLog {
    /// 创建带新 UUID 的日志节点。
    pub fn new() -> Self {
        Self {
            base: InstantEffectNodeBase::new(),
        }
    }
}

impl_effect_node_pin_group!(EffectNodeLog,
    input => (
        start => (message: String)
    )
    output => (
        finish => ()
    )
);

impl EffectNode for EffectNodeLog {}

impl InstantEffectNode for EffectNodeLog {
    fn get_uuid(&self) -> Uuid {
        self.base.node_id
    }

    fn collect(&self, _context: &mut EffectGraphContext) {}

    fn execute(&self, context: &mut EffectGraphContext) {
        info!(
            "node {} execute: {:?}",
            std::any::type_name::<EffectNodeLog>(),
            self.get_uuid(),
        );

        if let Some(slot) = self.get_input_slot_pin_by_name(EffectNodeLog::INPUT_SLOT_MESSAGE) {
            let message_value = context.get_input_value_type::<String>(&EffectNodeSlotPin {
                node_id: EffectNodeId::Uuid(self.get_uuid()),
                slot: *slot,
            });
            if let Some(value) = message_value {
                info!(
                    "node {} message: {}",
                    std::any::type_name::<EffectNodeLog>(),
                    value
                );
            }
        }
    }

    fn push_execute_chain(
        &self,
        context: &EffectGraphContext,
        executor: &mut crate::graph::executor::EffectGraphExecutor,
        _input_exec_pin: EffectNodeExec,
        instant_nodes: &Res<crate::graph::context::InstantEffectNodeMap>,
    ) {
        executor.continue_push_next_node_output_pin_from_node_name(
            self.get_uuid().into(),
            self,
            EffectNodeLog::OUTPUT_EXEC_FINISH,
            context,
            instant_nodes,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::blackboard::EffectValue;

    #[test]
    fn new_creates_node_with_non_nil_uuid() {
        let node = EffectNodeLog::new();
        assert!(!node.get_uuid().is_nil());
        assert_eq!(node.get_uuid(), node.base.node_id);
    }

    #[test]
    fn pin_groups_expose_start_input_with_message_slot() {
        let node = EffectNodeLog::new();

        assert_eq!(node.get_input_pin_group_num(), 1);
        assert_eq!(node.get_output_pin_group_num(), 1);

        let message_slot = node
            .get_input_slot_pin_by_name(EffectNodeLog::INPUT_SLOT_MESSAGE)
            .expect("message 输入槽应存在");
        assert_eq!(message_slot.pin_type, std::any::TypeId::of::<String>());
        assert_eq!(
            node.get_output_exec_pin_by_name(EffectNodeLog::OUTPUT_EXEC_FINISH)
                .map(|exec| exec.name),
            Some(EffectNodeLog::OUTPUT_EXEC_FINISH)
        );
    }

    #[test]
    fn pin_group_queries_missing_names_return_none() {
        let node = EffectNodeLog::new();
        assert_eq!(node.get_input_slot_pin_by_name("nonexistent"), None);
        assert_eq!(node.get_output_exec_pin_by_name("nonexistent"), None);
    }

    #[test]
    fn execute_reads_message_input_from_context() {
        let node = EffectNodeLog::new();
        let mut context = EffectGraphContext::new();
        let slot = node
            .get_input_slot_pin_by_name(EffectNodeLog::INPUT_SLOT_MESSAGE)
            .expect("message 输入槽应存在");
        context.insert_input_value(
            EffectNodeSlotPin {
                node_id: EffectNodeId::Uuid(node.get_uuid()),
                slot: *slot,
            },
            EffectValue::String("hello world".into()).into(),
        );

        node.execute(&mut context);
    }

    #[test]
    fn execute_without_message_input_does_not_panic() {
        let node = EffectNodeLog::new();
        let mut context = EffectGraphContext::new();

        node.execute(&mut context);
    }
}
