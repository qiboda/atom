// a skill
//     effect graph child
//         effect nodes children nodes.
//

use bevy::prelude::*;

use ability::nodes::{
    base::{
        entry::{EffectNodeEntry, EntryNodeBundle},
        msg::{EffectNodeMsg, MsgNodeBundle},
        timer::{EffectNodeTimer, TimerNodeBundle},
    },
    blackboard::EffectValue,
    build_graph,
    graph::{EffectGraph, EffectGraphBuilder, EffectGraphContext, EffectPinKey},
};

#[derive(Default)]
pub struct EffectNodeGraphBaseAttackPlugin {}

impl Plugin for EffectNodeGraphBaseAttackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, build_graph::<EffectNodeGraphBaseAttack>);
    }
}

#[derive(Debug, Component, Default)]
pub struct EffectNodeGraphBaseAttack {}

impl EffectGraphBuilder for EffectNodeGraphBaseAttack {
    fn build(
        &self,
        commands: &mut Commands,
        effect_graph_context: &mut EffectGraphContext,
        parent: Entity,
    ) {
        let entry_node = EntryNodeBundle::new();
        let entry_node_uuid = entry_node.base.uuid;
        let timer_node = TimerNodeBundle::new();
        let timer_node_uuid = timer_node.base.uuid;
        let msg_node = MsgNodeBundle::new();
        let msg_node_uuid = msg_node.effect_node_base.uuid;

        let msg_node_entity = commands.spawn(msg_node).set_parent(parent).id();
        let timer_node_entity = commands.spawn(timer_node).set_parent(parent).id();
        let entry_node_entity = commands.spawn(entry_node).set_parent(parent).id();

        effect_graph_context.entry_node = Some(entry_node_entity);

        effect_graph_context.insert_output_value(
            EffectPinKey {
                node: entry_node_entity,
                node_id: entry_node_uuid,
                key: EffectNodeEntry::OUTPUT_EXEC_FINISH,
            },
            EffectValue::Vec(vec![EffectValue::Entity(timer_node_entity)]),
        );

        effect_graph_context.outputs.insert(
            EffectPinKey {
                node: timer_node_entity,
                node_id: timer_node_uuid,
                key: EffectNodeTimer::OUTPUT_EXEC_FINISH,
            },
            EffectValue::Vec(vec![EffectValue::Entity(msg_node_entity)]),
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
