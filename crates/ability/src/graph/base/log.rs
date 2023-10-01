use std::any::TypeId;

use bevy::prelude::{info, App, Bundle, Component, Plugin};

use lazy_static::lazy_static;

use crate::graph::{
    blackboard::EffectValue,
    bundle::EffectNodeBaseBundle,
    context::EffectPinKey,
    event::{EffectEvent, EffectNodeEventPlugin},
    node::{
        EffectNode, EffectNodeExec, EffectNodeExecGroup, EffectNodePin, EffectNodePinGroup,
        EffectNodeStartContext, EffectNodeExecuteState, EffectNodeUuid, EffectNodeTickState, EffectNodeAbortContext,
    },
};

///////////////////////// Plugin /////////////////////////

#[derive(Debug, Default)]
pub struct EffectNodeLogPlugin {}

impl Plugin for EffectNodeLogPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EffectNodeEventPlugin::<EffectNodeLog>::default());
    }
}

impl EffectNodeLogPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

///////////////////////// Node Component /////////////////////////

#[derive(Debug, Default, Component)]
pub struct EffectNodeLog {}

impl EffectNodeLog {
    pub const INPUT_EXEC_START: &'static str = "start";
    pub const INPUT_PIN_MESSAGE: &'static str = "message";

    pub const OUTPUT_EXEC_FINISH: &'static str = "finish";
    pub const OUTPUT_PIN_MESSAGE: &'static str = "message";
}

impl EffectNodePinGroup for EffectNodeLog {
    fn get_input_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        lazy_static! {
            static ref INPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeLog::INPUT_EXEC_START
                },
                pins: vec![EffectNodePin {
                    name: EffectNodeLog::INPUT_PIN_MESSAGE,
                    pin_type: TypeId::of::<String>(),
                }],
            }];
        };
        &INPUT_PIN_GROUPS
    }

    fn get_output_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        lazy_static! {
            static ref OUTPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeLog::OUTPUT_EXEC_FINISH
                },
                pins: vec![],
            }];
        }
        &OUTPUT_PIN_GROUPS
    }
}

impl EffectNode for EffectNodeLog {
    fn start(&mut self, context: EffectNodeStartContext) {
        let duration_input_key = EffectPinKey {
            node: context.node_entity,
            node_id: *context.node_uuid,
            key: EffectNodeLog::INPUT_PIN_MESSAGE,
        };
        let duration_value = context.graph_context.get_input_value(&duration_input_key);

        if let Some(EffectValue::String(message)) = duration_value {
            info!("{}", message);
        }

        if let Some(EffectValue::Vec(entities)) =
            context.graph_context.get_output_value(&EffectPinKey {
                node: context.node_entity,
                node_id: *context.node_uuid,
                key: EffectNodeLog::OUTPUT_EXEC_FINISH,
            })
        {
            for entity in entities.iter() {
                if let EffectValue::Entity(entity) = entity {
                    context.event_writer.send(EffectEvent::Start(*entity));
                }
            }
        }
    }

    fn abort(&mut self, _context: EffectNodeAbortContext) {}
}

///////////////////////// Node Bundle /////////////////////////

#[derive(Bundle, Debug, Default)]
pub struct LogNodeBundle {
    pub effect_node: EffectNodeLog,
    pub effect_node_base: EffectNodeBaseBundle,
}

impl LogNodeBundle {
    pub fn new() -> Self {
        Self {
            effect_node: EffectNodeLog::default(),
            effect_node_base: EffectNodeBaseBundle {
                execute_state: EffectNodeExecuteState::default(),
                tick_state: EffectNodeTickState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}
