use std::any::TypeId;

use bevy::prelude::{
    default, info, App, Bundle, Component, Entity, EventWriter, Parent, Plugin, PreUpdate, Query,
    Update,
};

use lazy_static::lazy_static;

use crate::nodes::{
    blackboard::EffectValue,
    bundle::EffectNodeBaseBundle,
    event::EffectEvent,
    graph::{EffectGraphContext, EffectPinKey},
    node::{
        EffectNode, EffectNodeExec, EffectNodeExecGroup, EffectNodePin, EffectNodePinGroup,
        EffectNodeState, EffectNodeUuid,
    },
    receive_effect_event,
};

///////////////////////// Plugin /////////////////////////

#[derive(Debug, Default)]
pub struct EffectNodeMsgPlugin {}

impl Plugin for EffectNodeMsgPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, receive_effect_event::<EffectNodeMsg>)
            .add_systems(Update, update_msg);
    }
}

impl EffectNodeMsgPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

///////////////////////// Node Component /////////////////////////

#[derive(Debug, Default, Component)]
pub struct EffectNodeMsg {}

impl EffectNodeMsg {
    pub const INPUT_EXEC_START: &'static str = "start";
    pub const INPUT_PIN_MESSAGE: &'static str = "message";

    pub const OUTPUT_EXEC_FINISH: &'static str = "finish";
    pub const OUTPUT_PIN_MESSAGE: &'static str = "message";
}

impl EffectNodePinGroup for EffectNodeMsg {
    fn get_input_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        lazy_static! {
            static ref INPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeMsg::INPUT_EXEC_START
                },
                pins: vec![EffectNodePin {
                    name: EffectNodeMsg::INPUT_PIN_MESSAGE,
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
                    name: EffectNodeMsg::OUTPUT_EXEC_FINISH
                },
                pins: vec![],
            }];
        }
        &OUTPUT_PIN_GROUPS
    }
}

impl EffectNode for EffectNodeMsg {
    fn start(&mut self) {}

    fn clear(&mut self) {}

    fn abort(&mut self) {}

    fn update(&mut self) {}

    fn pause(&mut self) {}

    fn resume(&mut self) {}
}

///////////////////////// Node Bundle /////////////////////////

#[derive(Bundle, Debug)]
pub struct MsgNodeBundle {
    pub effect_node: EffectNodeMsg,
    pub effect_node_base: EffectNodeBaseBundle,
}

impl MsgNodeBundle {
    pub fn new() -> Self {
        Self {
            effect_node: EffectNodeMsg { ..default() },
            effect_node_base: EffectNodeBaseBundle {
                effect_node_state: EffectNodeState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}

impl Default for MsgNodeBundle {
    fn default() -> Self {
        Self {
            effect_node: EffectNodeMsg { ..default() },
            effect_node_base: EffectNodeBaseBundle::default(),
        }
    }
}

fn update_msg(
    mut query_graph: Query<&mut EffectGraphContext>,
    mut query: Query<(
        Entity,
        &EffectNodeMsg,
        &mut EffectNodeState,
        &EffectNodeUuid,
        &Parent,
    )>,
    mut event_writer: EventWriter<EffectEvent>,
) {
    for (entity, _msg, mut state, uuid, parent) in query.iter_mut() {
        if *state == EffectNodeState::Running {
            let graph_context = query_graph.get_mut(parent.get()).unwrap();

            let duration_input_key = EffectPinKey {
                node: entity,
                node_id: *uuid,
                key: EffectNodeMsg::INPUT_PIN_MESSAGE,
            };
            let duration_value = graph_context.get_input_value(&duration_input_key);

            if let Some(EffectValue::String(message)) = duration_value {
                info!("{}", message);
            }

            if let Some(EffectValue::Vec(entities)) =
                graph_context.get_output_value(&EffectPinKey {
                    node: entity,
                    node_id: *uuid,
                    key: EffectNodeMsg::OUTPUT_EXEC_FINISH,
                })
            {
                for entity in entities.iter() {
                    if let EffectValue::Entity(entity) = entity {
                        event_writer.send(EffectEvent::Start(*entity));
                    }
                }
            }

            *state = EffectNodeState::Finished;
        }
    }
}
