use std::any::TypeId;

use bevy::prelude::{
    default, info, App, Bundle, Component, EventWriter, Plugin, PreUpdate, Query, Update,
};

use crate::nodes::{
    bundle::EffectNodeBaseBundle,
    event::EffectEvent,
    node::{
        EffectNode, EffectNodeExec, EffectNodeExecGroup, EffectNodePin, EffectNodePinGroup,
        EffectNodeState, EffectNodeUuid,
    },
    receive_effect_event,
};

///////////////////////// Plugin /////////////////////////

#[derive(Debug)]
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
pub struct EffectNodeMsg {
    pub message: String,
}

impl EffectNodeMsg {
    const INPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![EffectNodeExecGroup {
        exec: EffectNodeExec { name: "start" },
        pins: vec![EffectNodePin {
            name: "message",
            pin_type: TypeId::of::<String>(),
        }],
    }];

    const OUTPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![EffectNodeExecGroup {
        exec: EffectNodeExec { name: "Finish" },
        pins: vec![],
    }];
}

impl EffectNodePinGroup for EffectNodeMsg {
    fn get_input_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        &Self::INPUT_PIN_GROUPS
    }

    fn get_output_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        &Self::OUTPUT_PIN_GROUPS
    }
}

impl EffectNode for EffectNodeMsg {
    fn start(&mut self) {
        todo!()
    }

    fn clear(&mut self) {
        todo!()
    }

    fn abort(&mut self) {
        todo!()
    }

    fn update(&mut self) {
        todo!()
    }

    fn pause(&mut self) {
        todo!()
    }

    fn resume(&mut self) {
        todo!()
    }
}

///////////////////////// Node Bundle /////////////////////////

#[derive(Bundle, Debug)]
pub struct MsgNodeBundle {
    effect_node: EffectNodeMsg,
    effect_node_base: EffectNodeBaseBundle,
}

impl MsgNodeBundle {
    pub fn new(message: &str) -> Self {
        Self {
            effect_node: EffectNodeMsg {
                message: message.to_string(),
                ..default()
            },
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
            effect_node: EffectNodeMsg {
                message: "hello".to_string(),
                ..default()
            },
            effect_node_base: EffectNodeBaseBundle::default(),
        }
    }
}

fn update_msg(
    mut query: Query<(&EffectNodeMsg, &mut EffectNodeState)>,
    mut event_writer: EventWriter<EffectEvent>,
) {
    for (msg, mut state) in query.iter_mut() {
        if *state == EffectNodeState::Running {
            info!("{}", msg.message);
            *state = EffectNodeState::Finished;

            for entity in msg.end_exec.entities.iter() {
                event_writer.send(EffectEvent::Start(*entity));
            }
        }
    }
}
