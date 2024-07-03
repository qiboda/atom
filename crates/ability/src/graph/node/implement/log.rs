use std::vec::Vec;

use bevy::prelude::*;
use uuid::Uuid;

use crate::{
    graph::{
        context::EffectGraphContext,
        node::{
            bundle::InstantEffectNodeBase,
            pin::{EffectNodeExec, EffectNodePinGroup},
            EffectNode, EffectNodeId, InstantEffectNode,
        },
        pin::EffectNodeSlotPin,
    },
    impl_effect_node_pin_group,
};

///////////////////////// Node Component /////////////////////////

#[derive(Debug, Default, Reflect)]
pub struct EffectNodeLog {
    pub base: InstantEffectNodeBase,
}

impl EffectNodeLog {
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

        let slot = self
            .get_input_slot_pin_by_name(EffectNodeLog::INPUT_SLOT_MESSAGE)
            .unwrap();
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
