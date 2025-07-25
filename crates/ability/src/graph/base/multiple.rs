use bevy::{prelude::*, reflect::Reflect};

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
pub struct EffectNodeMultiplePlugin {}

impl Plugin for EffectNodeMultiplePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                effect_node_start_event.run_if(node_can_start()),
                effect_node_pause_event::<EffectNodeMultiple>.run_if(node_can_pause()),
                effect_node_resume_event::<EffectNodeMultiple>.run_if(node_can_resume()),
            ),
        );
    }
}

///////////////////////// Node Event /////////////////////////

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Reflect)]
pub struct EffectNodeMultipleEndExec {
    pub entities: Vec<Entity>,
}

///////////////////////// Node Component /////////////////////////

#[derive(Debug, Default, Component, Reflect)]
pub struct EffectNodeMultiple {}

impl_effect_node_pin_group!(EffectNodeMultiple,
    input => (
        start, pins => (A: f32, B: f32)
    )
    output => (
        finish, pins => (C: f32)
    )
);

impl EffectNode for EffectNodeMultiple {}

///////////////////////// Node Bundle /////////////////////////

#[derive(Bundle, Debug, Default)]
pub struct MultipleNodeBundle {
    effect_node: EffectNodeMultiple,
    effect_node_base: EffectNodeBaseBundle,
}

impl MultipleNodeBundle {
    pub fn new() -> Self {
        Self {
            effect_node: EffectNodeMultiple::default(),
            effect_node_base: EffectNodeBaseBundle {
                execute_state: EffectNodeExecuteState::default(),
                tick_state: EffectNodeTickState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}

fn effect_node_start_event(
    mut query: Query<(&EffectNodeUuid, &Parent), With<EffectNodeMultiple>>,
    mut graph_query: Query<&mut EffectGraphContext>,
    mut event_writer: EventWriter<EffectNodeStartEvent>,
    pending: Res<EffectNodePendingEvents>,
) {
    for node_entity in pending.pending_start.iter() {
        if let Ok((node_uuid, parent)) = query.get_mut(*node_entity) {
            info!(
                "node {} start: {:?}",
                std::any::type_name::<EffectNodeMultiple>(),
                node_entity
            );

            let mut graph_context = graph_query.get_mut(parent.get()).unwrap();

            let a_value = graph_context.get_input_value(&EffectPinKey::new(
                *node_entity,
                *node_uuid,
                EffectNodeMultiple::INPUT_PIN_A,
            ));

            let b_value = graph_context.get_input_value(&EffectPinKey::new(
                *node_entity,
                *node_uuid,
                EffectNodeMultiple::INPUT_PIN_B,
            ));

            let mut c = EffectValue::F32(0.0);
            if let (Some(&EffectValue::F32(a)), Some(&EffectValue::F32(b))) = (a_value, b_value) {
                c = EffectValue::F32(a * b);
            }

            if let Some(c_value) = graph_context.get_output_value_mut(&EffectPinKey::new(
                *node_entity,
                *node_uuid,
                EffectNodeMultiple::OUTPUT_PIN_C,
            )) {
                *c_value = c;
            }

            graph_context.exec_next_nodes(
                *node_entity,
                *node_uuid,
                EffectNodeMultiple::OUTPUT_EXEC_FINISH,
                &mut event_writer,
            );
        }
    }
}
