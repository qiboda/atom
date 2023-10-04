use bevy::prelude::*;

use crate::{
    graph::{
        blackboard::EffectValue,
        bundle::EffectNodeBaseBundle,
        context::{EffectGraphContext, EffectPinKey},
        event::{
            effect_node_pause_event, effect_node_resume_event, node_can_pause, node_can_resume,
            node_can_start, EffectNodePendingEvents, EffectNodeStartEvent,
        },
        node::{EffectNode, EffectNodeExecuteState, EffectNodeTickState, EffectNodeUuid},
    },
    impl_effect_node_pin_group,
};

///////////////////////// Plugin /////////////////////////

#[derive(Debug, Default)]
pub struct EffectNodeLogPlugin {}

impl Plugin for EffectNodeLogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                effect_node_start_event.run_if(node_can_start()),
                effect_node_pause_event::<EffectNodeLog>.run_if(node_can_pause()),
                effect_node_resume_event::<EffectNodeLog>.run_if(node_can_resume()),
            ),
        );
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

impl_effect_node_pin_group!(EffectNodeLog,
    input => (
        start, pins => (message: String)
    )
    output => (
        finish, pins => ()
    )
);

impl EffectNode for EffectNodeLog {}

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

fn effect_node_start_event(
    mut query: Query<(&EffectNodeUuid, &Parent), With<EffectNodeLog>>,
    graph_query: Query<&EffectGraphContext>,
    mut event_writer: EventWriter<EffectNodeStartEvent>,
    pending: Res<EffectNodePendingEvents>,
) {
    for node_entity in pending.pending_start.iter() {
        if let Ok((node_uuid, parent)) = query.get_mut(*node_entity) {
            info!(
                "node {} start: {:?}",
                std::any::type_name::<EffectNodeLog>(),
                node_entity
            );

            let graph_context = graph_query.get(parent.get()).unwrap();

            let message_value = graph_context.get_input_value(&EffectPinKey::new(
                *node_entity,
                *node_uuid,
                EffectNodeLog::INPUT_PIN_MESSAGE,
            ));

            if let Some(EffectValue::String(message)) = message_value {
                info!("{}", message);
            }

            graph_context.exec_next_nodes(
                *node_entity,
                *node_uuid,
                EffectNodeLog::OUTPUT_EXEC_FINISH,
                &mut event_writer,
            );
        }
    }
}
