use bevy::prelude::*;

use crate::{
    graph::{
        blackboard::EffectValue,
        bundle::EffectNodeBaseBundle,
        context::{EffectGraphContext, EffectPinKey},
        event::{
            effect_node_pause_event, effect_node_resume_event, node_can_abort,
            node_can_check_start, node_can_pause, node_can_resume, node_can_start,
            EffectNodePendingEvents, EffectNodeStartEvent,
        },
        node::{EffectNode, EffectNodeExecuteState, EffectNodeTickState, EffectNodeUuid},
    },
    impl_effect_node_pin_group,
};

///////////////////////// Plugin /////////////////////////

#[derive(Debug, Default)]
pub struct EffectNodeEntryPlugin {}

impl Plugin for EffectNodeEntryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                effect_node_check_start_event.run_if(node_can_check_start()),
                effect_node_start_event.run_if(node_can_start()),
                effect_node_abort_event.run_if(node_can_abort()),
                effect_node_pause_event::<EffectNodeEntry>.run_if(node_can_pause()),
                effect_node_resume_event::<EffectNodeEntry>.run_if(node_can_resume()),
            ),
        );
    }
}

///////////////////////// Node Component /////////////////////////

#[derive(Debug, Default, Component)]
pub struct EffectNodeEntry;

impl_effect_node_pin_group!(EffectNodeEntry,
    output => (
        check_start, pins => (),
        start, pins => (),
        end, pins => (),
        abort, pins => ()
    )
);

impl EffectNode for EffectNodeEntry {}

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

fn effect_node_check_start_event(
    mut query: Query<(&EffectNodeUuid, &Parent), With<EffectNodeEntry>>,
    graph_query: Query<&EffectGraphContext>,
    pending: Res<EffectNodePendingEvents>,
    mut start_event_writer: EventWriter<EffectNodeStartEvent>,
) {
    for node_entity in pending.pending_check_can_start.iter() {
        if let Ok((node_uuid, parent)) = query.get_mut(*node_entity) {
            info!(
                "node {} check start: {:?}",
                std::any::type_name::<EffectNodeEntry>(),
                node_entity
            );

            let key = EffectPinKey {
                node: *node_entity,
                node_id: *node_uuid,
                key: EffectNodeEntry::OUTPUT_EXEC_CHECK_START,
            };
            let graph_context = graph_query.get(parent.get()).unwrap();
            if let Some(EffectValue::Vec(entities)) = graph_context.get_output_value(&key) {
                for entity in entities.iter() {
                    if let EffectValue::Entity(entity) = entity {
                        start_event_writer.send(EffectNodeStartEvent::new(*entity));
                    }
                }
            }
        }
    }
}

fn effect_node_start_event(
    mut query: Query<(&EffectNodeUuid, &Parent), With<EffectNodeEntry>>,
    graph_query: Query<&EffectGraphContext>,
    mut events: EventWriter<EffectNodeStartEvent>,
    pendig: Res<EffectNodePendingEvents>,
) {
    for entry in pendig.pending_start.iter() {
        if let Ok((node_uuid, parent)) = query.get_mut(*entry) {
            info!(
                "node {} start: {:?}",
                std::any::type_name::<EffectNodeEntry>(),
                entry
            );
            let graph_context = graph_query.get(parent.get()).unwrap();
            let key = EffectPinKey {
                node: *entry,
                node_id: *node_uuid,
                key: EffectNodeEntry::OUTPUT_EXEC_START,
            };
            if let Some(EffectValue::Vec(entities)) = graph_context.get_output_value(&key) {
                for entity in entities.iter() {
                    if let EffectValue::Entity(entity) = entity {
                        events.send(EffectNodeStartEvent::new(*entity));
                    }
                }
            }
        }
    }
}

fn effect_node_abort_event(
    mut query: Query<(&EffectNodeUuid, &Parent), With<EffectNodeEntry>>,
    graph_query: Query<&EffectGraphContext>,
    pending: Res<EffectNodePendingEvents>,
    mut start_event_writer: EventWriter<EffectNodeStartEvent>,
) {
    for node_entity in pending.pending_abort.iter() {
        if let Ok((node_uuid, parent)) = query.get_mut(*node_entity) {
            info!(
                "node {} resume: {:?}",
                std::any::type_name::<EffectNodeEntry>(),
                node_entity
            );
            let graph_context = graph_query.get(parent.get()).unwrap();
            let key = EffectPinKey {
                node: *node_entity,
                node_id: *node_uuid,
                key: EffectNodeEntry::OUTPUT_EXEC_ABORT,
            };
            if let Some(EffectValue::Vec(entities)) = graph_context.get_output_value(&key) {
                for entity in entities.iter() {
                    if let EffectValue::Entity(entity) = entity {
                        start_event_writer.send(EffectNodeStartEvent::new(*entity));
                    }
                }
            }
        }
    }
}
