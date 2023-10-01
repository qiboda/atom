use std::any::TypeId;

use bevy::prelude::{
    info, App, Bundle, Commands, Component, Entity, Plugin, EventWriter,
};

use lazy_static::lazy_static;

use crate::graph::{
    blackboard::EffectValue,
    bundle::EffectNodeBaseBundle,
    context::{EffectGraphContext, EffectPinKey},
    event::{EffectEvent, EffectNodeEventPlugin},
    node::{
        EffectNode, EffectNodeExec, EffectNodeExecGroup, EffectNodePin, EffectNodePinGroup,
        EffectNodeState, EffectNodeUuid,
    },
};

///////////////////////// Plugin /////////////////////////

#[derive(Debug, Default)]
pub struct EffectNodeLogPlugin {}

impl Plugin for EffectNodeLogPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EffectNodeEventPlugin::<EffectNodeLog>::default())
            // .add_systems(Update, update_msg);
            ;
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
    fn start(
        &mut self,
        _commands: &mut Commands,
        node_entity: Entity,
        node_uuid: &EffectNodeUuid,
        _node_state: &mut EffectNodeState,
        graph_context: &mut EffectGraphContext,
        event_writer: &mut EventWriter<EffectEvent>,
    ) {
        let duration_input_key = EffectPinKey {
            node: node_entity,
            node_id: *node_uuid,
            key: EffectNodeLog::INPUT_PIN_MESSAGE,
        };
        let duration_value = graph_context.get_input_value(&duration_input_key);

        if let Some(EffectValue::String(message)) = duration_value {
            info!("{}", message);
        }

        if let Some(EffectValue::Vec(entities)) = graph_context.get_output_value(&EffectPinKey {
            node: node_entity,
            node_id: *node_uuid,
            key: EffectNodeLog::OUTPUT_EXEC_FINISH,
        }) {
            for entity in entities.iter() {
                if let EffectValue::Entity(entity) = entity {
                    event_writer.send(EffectEvent::Start(*entity));
                }
            }
        }
    }

    fn clear(&mut self) {}

    fn abort(&mut self) {}

    fn update(&mut self) {}

    fn pause(&mut self) {}

    fn resume(&mut self) {}
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
                effect_node_state: EffectNodeState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}

// fn update_msg(
//     mut query_graph: Query<&mut EffectGraphContext>,
//     mut query: Query<(
//         Entity,
//         &EffectNodeMsg,
//         &mut EffectNodeState,
//         &EffectNodeUuid,
//         &Parent,
//     )>,
//     mut event_writer: EventWriter<EffectEvent>,
// ) {
//     for (entity, _msg, mut state, uuid, parent) in query.iter_mut() {
//         if *state == EffectNodeState::Running {
//             let graph_context = query_graph.get_mut(parent.get()).unwrap();
//         }
//     }
// }
