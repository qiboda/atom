use bevy::{
    prelude::{
        App, Bundle, Component, Entity, EventWriter, Parent, Plugin, PreUpdate, Query, Res, Update,
    },
    time::Time,
};

use lazy_static::lazy_static;

use crate::nodes::{
    blackboard::EffectValue,
    bundle::EffectNodeBaseBundle,
    event::EffectEvent,
    node::{
        EffectDynamicNode, EffectNode, EffectNodeExec, EffectNodeExecGroup, EffectNodePin,
        EffectNodePinGroup, EffectNodeState, EffectNodeUuid,
    },
    receive_effect_event, graph::{EffectGraphContext, EffectPinKey},
};

#[derive(Debug)]
pub struct EffectNodeTimerPlugin;

impl Plugin for EffectNodeTimerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, receive_effect_event::<EffectNodeTimer>)
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
                effect_node_state: EffectNodeState::default(),
                uuid: EffectNodeUuid::new(),
            },
        }
    }
}

#[derive(Clone, Debug, Default, Component)]
pub struct EffectNodeTimer {
    pub elapse: f32,
}

impl EffectNode for EffectNodeTimer {
    fn start(&mut self) {
        self.elapse = 0.0;
    }

    fn clear(&mut self) {
        self.elapse = 0.0;
    }

    fn abort(&mut self) {
        self.clear();
    }

    fn pause(&mut self) {}

    fn resume(&mut self) {}

    fn update(&mut self) {}
}

impl EffectDynamicNode for EffectNodeTimer {}

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
        &mut EffectNodeState,
        &EffectNodeUuid,
        &Parent,
    )>,
    mut event_writer: EventWriter<EffectEvent>,
    time: Res<Time>,
) {
    for (entity, mut timer, mut state, uuid, parent) in query.iter_mut() {
        if *state == EffectNodeState::Running {
            let elapse = timer.elapse + time.delta_seconds();
            timer.elapse = elapse;

            let graph_context = graph_query.get(parent.get()).unwrap();

            let input_key = EffectPinKey {
                node: entity,
                node_id: *uuid,
                key: EffectNodeTimer::INPUT_PIN_DURATION,
            };
            if let Some(EffectValue::F32(duration_value)) =
                graph_context.get_input_value(&input_key)
            {
                if timer.elapse >= *duration_value {
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

                    *state = EffectNodeState::Finished;
                }
            }
        }
    }
}
