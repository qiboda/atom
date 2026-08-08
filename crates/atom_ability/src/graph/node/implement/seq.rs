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
