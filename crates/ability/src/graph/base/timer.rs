use std::ops::Not;

use bevy::{
    prelude::{App, Bundle, Component, Entity, EventWriter, Parent, Plugin, Query, Res, Update},
    time::Time,
};

use lazy_static::lazy_static;

use crate::graph::{
    blackboard::EffectValue,
    bundle::EffectNodeBaseBundle,
    context::{EffectGraphContext, EffectPinKey},
    event::{EffectEvent, EffectNodeEventPlugin},
    node::{
        EffectNode, EffectNodeAbortContext, EffectNodeExec, EffectNodeExecGroup,
        EffectNodeExecuteState, EffectNodePin, EffectNodePinGroup, EffectNodeStartContext,
        EffectNodeTickState, EffectNodeUuid,
    },
};

#[derive(Debug)]
pub struct EffectNodeTimerPlugin;

impl Plugin for EffectNodeTimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EffectNodeEventPlugin::<EffectNodeTimer>::default())
            .add_systems(Update, update_timer);
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

impl EffectNode for EffectNodeTimer {
    fn start(&mut self, context: EffectNodeStartContext) {
        self.elapse.push(0.0);
        if self.elapse.is_empty().not() {
            *context.node_state = EffectNodeExecuteState::Actived;
        }
    }

    fn abort(&mut self, _context: EffectNodeAbortContext) {}
}

impl EffectNodeTimer {
    pub const INPUT_EXEC_START: &'static str = "start";
    pub const INPUT_PIN_DURATION: &'static str = "duration";

    pub const OUTPUT_EXEC_FINISH: &'static str = "finish";
}

impl EffectNodePinGroup for EffectNodeTimer {
    fn get_input_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        lazy_static! {
            static ref INPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeTimer::INPUT_EXEC_START,
                },
                pins: vec![EffectNodePin {
                    name: EffectNodeTimer::INPUT_PIN_DURATION,
                    pin_type: std::any::TypeId::of::<f32>(),
                }],
            }];
        }
        &INPUT_PIN_GROUPS
    }

    fn get_output_pin_group(&self) -> &Vec<EffectNodeExecGroup> {
        lazy_static! {
            static ref OUTPUT_PIN_GROUPS: Vec<EffectNodeExecGroup> = vec![EffectNodeExecGroup {
                exec: EffectNodeExec {
                    name: EffectNodeTimer::OUTPUT_EXEC_FINISH,
                },
                pins: vec![],
            }];
        }
        &OUTPUT_PIN_GROUPS
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
    mut event_writer: EventWriter<EffectEvent>,
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
                                event_writer.send(EffectEvent::Start(*entity));
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
