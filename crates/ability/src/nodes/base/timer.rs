use bevy::{
    prelude::{App, Bundle, Component, Entity, EventWriter, Plugin, PreUpdate, Query, Res, Update},
    time::Time,
};

use lazy_static::lazy_static;

use crate::nodes::{
    bundle::EffectNodeBaseBundle,
    event::EffectEvent,
    node::{
        EffectDynamicNode, EffectNode, EffectNodeExec, EffectNodeExecGroup, EffectNodePin,
        EffectNodePinGroup, EffectNodeState, EffectNodeUuid,
    },
    receive_effect_event,
};

#[derive(Debug)]
pub struct EffectNodeTimerPluign;

impl Plugin for EffectNodeTimerPluign {
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
    pub fn new(duration: f32) -> Self {
        Self {
            timer: EffectNodeTimer {
                elapse: 0.0,
                duration,
                ..Default::default()
            },
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
    pub duration: f32,
}

impl EffectNode for EffectNodeTimer {
    fn start(&mut self) {
        self.elapse = 0.0;
    }

    fn clear(&mut self) {
        self.elapse = 0.0;
        self.duration = f32::MAX;
    }

    fn abort(&mut self) {
        self.clear();
    }

    fn pause(&mut self) {
        todo!()
    }

    fn resume(&mut self) {
        todo!()
    }

    fn update(&mut self) {}
}

impl EffectDynamicNode for EffectNodeTimer {}

impl EffectNodeTimer {
    pub const INPUT_EXEC_START: &'static str = "start";
    pub const INPUT_PIN_DURATION: &'static str = "duration";
    pub const OUTPUT_EXEC_FINISHED: &'static str = "finished";
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
                    name: EffectNodeTimer::OUTPUT_EXEC_FINISHED,
                },
                pins: vec![],
            }];
        }
        &OUTPUT_PIN_GROUPS
    }
}

fn update_timer(
    mut query: Query<(&mut EffectNodeTimer, &mut EffectNodeState)>,
    mut event_writer: EventWriter<EffectEvent>,
    time: Res<Time>,
) {
    for (mut timer, mut state) in query.iter_mut() {
        match *state {
            EffectNodeState::Running => {
                timer.elapse += time.delta_seconds();
                if timer.elapse >= timer.duration {
                    *state = EffectNodeState::Finished;
                    // for entity in timer.end_exec.entities.iter() {
                    //     event_writer.send(EffectEvent::Start(*entity));
                    // }
                }
            }
            _ => {}
        }
    }
}
