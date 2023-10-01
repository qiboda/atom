use bevy::prelude::{
    App, Bundle, Commands, Component, Entity, EventWriter, Plugin
};

use lazy_static::lazy_static;

use crate::graph::{
    blackboard::EffectValue,
    bundle::EffectNodeBaseBundle,
    context::{EffectGraphContext, EffectPinKey},
    event::{EffectEvent, EffectNodeEventPlugin},
    node::{
        EffectNode, EffectNodeExec, EffectNodeExecGroup, EffectNodePinGroup, EffectNodeState,
        EffectNodeUuid,
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
    fn start(
        &mut self,
        _commands: &mut Commands,
        node_entity: Entity,
        node_uuid: &EffectNodeUuid,
        _node_state: &mut EffectNodeState,
        graph_context: &mut EffectGraphContext,
        event_writer: &mut EventWriter<EffectEvent>,
    ) {
        let key = EffectPinKey {
            node: node_entity,
            node_id: *node_uuid,
            key: EffectNodeEntry::OUTPUT_EXEC_FINISH,
        };
        if let Some(EffectValue::Vec(entities)) = graph_context.get_output_value(&key) {
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
pub struct EntryNodeBundle {
    pub node: EffectNodeEntry,
    pub base: EffectNodeBaseBundle,
}

impl EntryNodeBundle {
    pub fn new() -> Self {
        Self {
            node: EffectNodeEntry,
            base: EffectNodeBaseBundle {
                effect_node_state: EffectNodeState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}

// fn update_entry(
//     mut query_graph: Query<&mut EffectGraphContext>,
//     mut query: Query<(
//         Entity,
//         &EffectNodeEntry,
//         &mut EffectNodeState,
//         &EffectNodeUuid,
//         &Parent,
//     )>,
//     mut event_writer: EventWriter<EffectEvent>,
// ) {
//     for (entity, _entry, mut state, uuid, parent) in query.iter_mut() {
//         if *state == EffectNodeState::Running {}
//     }
// }
