// a skill
//     effect graph child
//         effect nodes children nodes.
//

use std::sync::Arc;

use bevy::prelude::*;

use atom_ability::{
    ability::node::ability_entry::EffectNodeAbilityEntry,
    graph::{
        blackboard::EffectValue,
        builder::{EffectGraph, EffectGraphBuilder},
        bundle::EffectGraphBundle,
        context::{EffectGraphContext, GraphRef, InstantEffectNodeMap},
        node::{
            InstantEffectNode,
            bundle::StateEffectNodeBundle,
            implement::{log::EffectNodeLog, timer::EffectNodeTimer},
            pin::EffectNodePinGroup,
        },
        pin::{EffectNodeExecPin, EffectNodeSlotPin},
        state::{EffectGraphState, EffectGraphTickState},
    },
};

#[derive(Debug, Component, Default, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct EffectNodeGraphBaseAttack;

impl EffectGraphBuilder for EffectNodeGraphBaseAttack {
    fn get_effect_graph_name(&self) -> &'static str {
        "EffectNodeGraphBaseAttack"
    }

    fn build(
        &self,
        commands: &mut Commands,
        instant_map: &mut ResMut<InstantEffectNodeMap>,
    ) -> Entity {
        // 不需要运行时数据的存储到技能下，作为技能内的单例，
        //      不使用system
        // 需要运行时数据的存储到Entity上，作为组件存在。
        //      使用system
        //      不使用system
        // 执行器，获取下一个节点，非system节点，都不需要存储数据，因此都是单例。
        //      因此可以存储在ability下，不作为entity存在。
        //
        // 构建所需要做的事情
        // create node
        // set node input value
        // node exec to node exec
        // node output to node input(type check)

        let mut context = EffectGraphContext::new();

        // 创建节点
        let mut entry_node = StateEffectNodeBundle::<EffectNodeAbilityEntry>::default();
        let entry_node_entity = commands.spawn_empty().id();
        entry_node.node_id = entry_node_entity.into();
        context.insert_state_node(entry_node_entity);
        context.set_entry_node(entry_node_entity);

        let mut timer_node = StateEffectNodeBundle::<EffectNodeTimer>::default();
        let timer_node_entity = commands.spawn_empty().id();
        timer_node.node_id = timer_node_entity.into();
        if let Some(slot) = timer_node
            .state_node
            .get_input_slot_pin_by_name(EffectNodeTimer::INPUT_SLOT_DURATION)
        {
            context.insert_input_value(
                EffectNodeSlotPin {
                    node_id: timer_node_entity.into(),
                    slot: *slot,
                },
                EffectValue::F32(5.0).into(),
            );
        }
        context.insert_state_node(timer_node_entity);

        let log_node = EffectNodeLog::new();
        if let Some(slot) = log_node.get_input_slot_pin_by_name(EffectNodeLog::INPUT_SLOT_MESSAGE) {
            context.insert_input_value(
                EffectNodeSlotPin {
                    node_id: log_node.get_uuid().into(),
                    slot: *slot,
                },
                EffectValue::String("msg output".into()).into(),
            );
        }
        context.insert_instant_node(log_node.get_uuid());

        let entry_output_start = entry_node
            .state_node
            .get_output_exec_pin_by_name(EffectNodeAbilityEntry::OUTPUT_EXEC_START);
        let timer_input_start = timer_node
            .state_node
            .get_input_exec_pin_by_name(EffectNodeTimer::INPUT_EXEC_START);

        if let (Some(entry_slot), Some(timer_slot)) = (entry_output_start, timer_input_start) {
            context.add_exec_connection(
                EffectNodeExecPin {
                    node_id: entry_node.node_id,
                    exec: *entry_slot,
                },
                &[EffectNodeExecPin {
                    node_id: timer_node.node_id,
                    exec: *timer_slot,
                }],
            );
        }
        let timer_output_finish = timer_node
            .state_node
            .get_output_exec_pin_by_name(EffectNodeTimer::OUTPUT_EXEC_FINISH);

        let log_input_start = log_node.get_input_exec_pin_by_name(EffectNodeLog::INPUT_EXEC_START);
        if let (Some(timer_slot), Some(log_slot)) = (timer_output_finish, log_input_start) {
            context.add_exec_connection(
                EffectNodeExecPin {
                    node_id: timer_node.node_id,
                    exec: *timer_slot,
                },
                &[EffectNodeExecPin {
                    node_id: log_node.get_uuid().into(),
                    exec: *log_slot,
                }],
            );
        }

        let mut parent_commands = commands.spawn_empty();
        context.set_graph_ref(GraphRef::new(parent_commands.id()));
        let parent = parent_commands
            .insert(EffectGraphBundle {
                context,
                state: EffectGraphState::Inactive,
                graph: *self,
                tick_state: EffectGraphTickState::Ticked,
            })
            .id();
        commands
            .entity(entry_node_entity)
            .insert(entry_node)
            .set_parent_in_place(parent);
        commands
            .entity(timer_node_entity)
            .insert(timer_node)
            .set_parent_in_place(parent);
        instant_map.insert(log_node.get_uuid(), Arc::new(log_node));
        parent
    }
}

impl EffectGraph for EffectNodeGraphBaseAttack {}
