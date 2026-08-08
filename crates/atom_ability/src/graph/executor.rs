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
