use std::any::TypeId;

use bevy::prelude::{
    default, info, App, Bundle, Component, EventWriter, Plugin, PreUpdate, Query, Update,
};

use lazy_static::lazy_static;

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
pub struct EffectNodeEntryPlugin {}

impl Plugin for EffectNodeEntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, receive_effect_event::<EffectNodeEntry>)
            .add_systems(Update, update_entry);
    }
}

impl EffectNodeEntryPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

///////////////////////// Node Component /////////////////////////

#[derive(Debug, Default, Component)]
pub struct EffectNodeEntry {}

impl EffectNodeEntry {
    pub const OUTPUT_EXEC_FINISH: &'static str = "finish";
}

impl EffectNodePinGroup for EffectNodeEntry {
    fn get_input_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        lazy_static! {
            static ref INPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![];
        };
        &INPUT_PIN_GROUPS
    }

    fn get_output_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        lazy_static! {
            static ref OUTPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeEntry::OUTPUT_EXEC_FINISH
                },
                pins: vec![],
            }];
        }
        &OUTPUT_PIN_GROUPS
    }
}

impl EffectNode for EffectNodeEntry {
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
pub struct EntryNodeBundle {
    pub node: EffectNodeEntry,
    pub base: EffectNodeBaseBundle,
}

impl EntryNodeBundle {
    pub fn new() -> Self {
        Self {
            node: EffectNodeEntry { ..default() },
            base: EffectNodeBaseBundle {
                effect_node_state: EffectNodeState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}

impl Default for EntryNodeBundle {
    fn default() -> Self {
        Self {
            node: EffectNodeEntry { ..default() },
            base: EffectNodeBaseBundle::default(),
        }
    }
}

fn update_entry(
    mut query: Query<(&EffectNodeEntry, &mut EffectNodeState)>,
    mut event_writer: EventWriter<EffectEvent>,
) {
    for (msg, mut state) in query.iter_mut() {
        if *state == EffectNodeState::Running {
            *state = EffectNodeState::Finished;

            // for entity in msg.end_exec.entities.iter() {
            //     event_writer.send(EffectEvent::Start(*entity));
            // }
        }
    }
}
