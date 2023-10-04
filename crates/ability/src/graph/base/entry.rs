use bevy::prelude::*;

use crate::{
    graph::{
        bundle::EffectNodeBaseBundle,
        context::EffectGraphContext,
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

            let graph_context = graph_query.get(parent.get()).unwrap();

            graph_context.exec_next_nodes(
                *node_entity,
                *node_uuid,
                EffectNodeEntry::OUTPUT_EXEC_CHECK_START,
                &mut start_event_writer,
            );
        }
    }
}

fn effect_node_start_event(
    mut query: Query<(&EffectNodeUuid, &Parent), With<EffectNodeEntry>>,
    graph_query: Query<&EffectGraphContext>,
    mut event_writer: EventWriter<EffectNodeStartEvent>,
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
            graph_context.exec_next_nodes(
                *entry,
                *node_uuid,
                EffectNodeEntry::OUTPUT_EXEC_START,
                &mut event_writer,
            );
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
            graph_context.exec_next_nodes(
                *node_entity,
                *node_uuid,
                EffectNodeEntry::OUTPUT_EXEC_ABORT,
                &mut start_event_writer,
            );
        }
    }
}
