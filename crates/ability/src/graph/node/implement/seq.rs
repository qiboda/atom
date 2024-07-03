use std::vec::Vec;

use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    graph::{
        context::{EffectGraphContext, InstantEffectNodeMap},
        executor::EffectGraphExecutor,
        node::{bundle::InstantEffectNodeBase, pin::EffectNodeExec, EffectNode, InstantEffectNode},
    },
    impl_effect_node_pin_group,
};

///////////////////////// Node Component /////////////////////////

#[derive(Debug, Default, Reflect)]
pub struct EffectNodeSeq {
    pub base: InstantEffectNodeBase,
}

impl EffectNodeSeq {
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
