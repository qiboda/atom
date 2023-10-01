use bevy::prelude::{App, Bundle, Component, Plugin};

use lazy_static::lazy_static;

use crate::graph::{
    blackboard::EffectValue,
    bundle::EffectNodeBaseBundle,
    context::EffectPinKey,
    event::{EffectEvent, EffectNodeEventPlugin},
    node::{
        EffectNode, EffectNodeAbortContext, EffectNodeExec, EffectNodeExecGroup,
        EffectNodeExecuteState, EffectNodePinGroup, EffectNodeStartContext, EffectNodeTickState, EffectNodeUuid,
    },
};

///////////////////////// Plugin /////////////////////////

#[derive(Debug, Default)]
pub struct EffectNodeEntryPlugin {}

impl Plugin for EffectNodeEntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EffectNodeEventPlugin::<EffectNodeEntry>::default());
    }
}

///////////////////////// Node Component /////////////////////////

#[derive(Debug, Default, Component)]
pub struct EffectNodeEntry;

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
    fn start(&mut self, context: EffectNodeStartContext) {
        let key = EffectPinKey {
            node: context.node_entity,
            node_id: *context.node_uuid,
            key: EffectNodeEntry::OUTPUT_EXEC_FINISH,
        };
        if let Some(EffectValue::Vec(entities)) = context.graph_context.get_output_value(&key) {
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
pub struct EntryNodeBundle {
    pub node: EffectNodeEntry,
    pub base: EffectNodeBaseBundle,
}

impl EntryNodeBundle {
    pub fn new() -> Self {
        Self {
            node: EffectNodeEntry,
            base: EffectNodeBaseBundle {
                execute_state: EffectNodeExecuteState::default(),
                tick_state: EffectNodeTickState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}
