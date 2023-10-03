use std::ops::Not;

use bevy::{prelude::*, time::Time};

use once_cell::sync::OnceCell;

use crate::graph::{
    blackboard::EffectValue,
    bundle::EffectNodeBaseBundle,
    context::{EffectGraphContext, EffectPinKey},
    event::{
        effect_node_pause_event, effect_node_resume_event, node_can_pause, node_can_resume,
        node_can_start, EffectNodePendingEvents, EffectNodeStartEvent,
    },
    node::{
        EffectNode, EffectNodeExec, EffectNodeExecGroup, EffectNodeExecuteState, EffectNodePin,
        EffectNodePinGroup, EffectNodeTickState, EffectNodeUuid,
    },
};

#[derive(Debug)]
pub struct EffectNodeTimerPlugin;

impl Plugin for EffectNodeTimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_timer).add_systems(
            Update,
            (
                effect_node_start_event.run_if(node_can_start()),
                effect_node_pause_event::<EffectNodeTimer>.run_if(node_can_pause()),
                effect_node_resume_event::<EffectNodeTimer>.run_if(node_can_resume()),
            ),
        );
    }
}

#[derive(Bundle, Debug, Default)]
pub struct TimerNodeBundle {
    pub timer: EffectNodeTimer,
    pub base: EffectNodeBaseBundle,
}

impl TimerNodeBundle {
    pub fn new() -> Self {
        Self {
            timer: EffectNodeTimer::default(),
            base: EffectNodeBaseBundle {
                execute_state: EffectNodeExecuteState::default(),
                tick_state: EffectNodeTickState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}

#[derive(Clone, Debug, Default, Component)]
pub struct EffectNodeTimer {
    pub elapse: Vec<f32>,
}

impl EffectNode for EffectNodeTimer {}

impl EffectNodeTimer {
    pub const INPUT_EXEC_START: &'static str = "start";
    pub const INPUT_PIN_DURATION: &'static str = "duration";

    pub const OUTPUT_EXEC_FINISH: &'static str = "finish";
}

impl EffectNodePinGroup for EffectNodeTimer {
    fn get_input_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        static CELL: OnceCell<Vec<EffectNodeExecGroup>> = OnceCell::new();
        CELL.get_or_init(|| {
            vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeTimer::INPUT_EXEC_START,
                },
                pins: vec![EffectNodePin {
                    name: EffectNodeTimer::INPUT_PIN_DURATION,
                    pin_type: std::any::TypeId::of::<f32>(),
                }],
            }]
        })
    }

    fn get_output_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        static CELL: OnceCell<Vec<EffectNodeExecGroup>> = OnceCell::new();
        CELL.get_or_init(|| {
            vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeTimer::OUTPUT_EXEC_FINISH,
                },
                pins: vec![],
            }]
        })
    }
}

fn effect_node_start_event(
    mut query: Query<(&mut EffectNodeTimer, &mut EffectNodeExecuteState)>,
    pending: Res<EffectNodePendingEvents>,
) {
    for node_entity in pending.pending_start.iter() {
        if let Ok((mut node, mut state)) = query.get_mut(*node_entity) {
            info!(
                "node {} start: {:?}",
                std::any::type_name::<EffectNodeTimer>(),
                node_entity
            );

            node.elapse.push(0.0);
            if node.elapse.is_empty().not() {
                *state = EffectNodeExecuteState::Actived;
            }
        }
    }
}

fn update_timer(
    graph_query: Query<&EffectGraphContext>,
    mut query: Query<(
        Entity,
        &mut EffectNodeTimer,
        &mut EffectNodeExecuteState,
        &EffectNodeUuid,
        &Parent,
    )>,
    mut event_writer: EventWriter<EffectNodeStartEvent>,
    time: Res<Time>,
) {
    for (entity, mut timer, mut node_state, uuid, parent) in query.iter_mut() {
        if *node_state == EffectNodeExecuteState::Idle {
            continue;
        }

        // set idle state before remove effect(delay a frame to set idle),
        // avoid to graph be removed before next node active.
        if timer.elapse.is_empty() {
            *node_state = EffectNodeExecuteState::Idle;
            return;
        }

        let graph_context = graph_query.get(parent.get()).unwrap();

        let input_key = EffectPinKey {
            node: entity,
            node_id: *uuid,
            key: EffectNodeTimer::INPUT_PIN_DURATION,
        };
        if let Some(EffectValue::F32(duration_value)) = graph_context.get_input_value(&input_key) {
            timer
                .elapse
                .iter_mut()
                .for_each(|x| *x += time.delta_seconds());

            timer.elapse.retain(|x| {
                if x >= duration_value {
                    if let Some(EffectValue::Vec(entities)) =
                        graph_context.get_output_value(&EffectPinKey {
                            node: entity,
                            node_id: *uuid,
                            key: EffectNodeTimer::OUTPUT_EXEC_FINISH,
                        })
                    {
                        for entity in entities.iter() {
                            if let EffectValue::Entity(entity) = entity {
                                event_writer.send(EffectNodeStartEvent::new(*entity));
                            }
                        }
                    }
                    return false;
                }
                true
            });
        }
    }
}
