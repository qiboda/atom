// a skill
//     effect graph child
//         effect nodes children nodes.
//

use bevy::prelude::*;

use crate::nodes::{
    base::{
        entry::EntryNodeBundle,
        msg::{ MsgNodeBundle, EffectNodeMsg } ,
        timer::{EffectNodeTimer, TimerNodeBundle},
    },
    blackboard::EffectValue,
    graph::{EffectGraph, EffectGraphBuilder, EffectGraphContext, EffectPinKey},
};

#[derive(Debug, Component)]
pub struct EffectNodeGraphBaseAttack {}

impl EffectGraphBuilder for EffectNodeGraphBaseAttack {
    fn build(&self, commands: &mut Commands, effect_graph_context: &mut EffectGraphContext) {
        let entry_node = EntryNodeBundle::new();
        let entry_node_uuid = entry_node.base.uuid;
        let timer_node = TimerNodeBundle::new(3.0);
        let timer_node_uuid = timer_node.base.uuid;
        let msg_node = MsgNodeBundle::new();
        let msg_node_uuid = msg_node.effect_node_base.uuid;

        let msg_node_entity = commands.spawn(msg_node).id();
        let timer_node_entity = commands.spawn(timer_node).id();
        let entry_node_entity = commands.spawn(entry_node).id();

        effect_graph_context.insert_output_value(
            EffectPinKey {
                node: entry_node_entity,
                node_id: entry_node_uuid,
                key: EffectNodeTimer::OUTPUT_EXEC_FINISHED,
            },
            EffectValue::Vec(
                vec![EffectValue::Entity(timer_node_entity)]
                    .into_iter()
                    .collect(),
            ),
        );

        effect_graph_context.outputs.insert(
            EffectPinKey {
                node: timer_node_entity,
                node_id: timer_node_uuid,
                key: EffectNodeTimer::OUTPUT_EXEC_FINISHED,
            },
            EffectValue::Vec(
                vec![EffectValue::Entity(msg_node_entity)]
                    .into_iter()
                    .collect(),
            ),
        );
        effect_graph_context.inputs.insert(
            EffectPinKey {
                node: timer_node_entity,
                node_id: timer_node_uuid,
                key: EffectNodeTimer::INPUT_PIN_DURATION,
            },
            EffectValue::F32(5.0),
        );
        effect_graph_context.inputs.insert(
            EffectPinKey {
                node: msg_node_entity,
                node_id: msg_node_uuid,
                key: EffectNodeMsg::INPUT_PIN_MESSAGE,
            },
            EffectValue::String("message log".into()),
        );
    }
}

impl EffectGraph for EffectNodeGraphBaseAttack {}
