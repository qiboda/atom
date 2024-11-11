use bevy::{
    prelude::{Commands, Component, Entity, Query, Res, With},
    reflect::Reflect,
    time::{Time, Timer, TimerMode},
};

use crate::graph::{event::EffectGraphExecEvent, state::EffectGraphState};

use super::{node::buff_entry::EffectNodeBuffEntry, state::Buff};

#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct BuffTime {
    pub once_timer: Timer,
    pub looper_timer: Option<Timer>,
}

impl BuffTime {
    pub fn new(once_duration: f32, looper_duration: Option<f32>) -> Self {
        Self {
            once_timer: Timer::from_seconds(once_duration, TimerMode::Once),
            looper_timer: looper_duration.map(|x| Timer::from_seconds(x, TimerMode::Repeating)),
        }
    }
}

pub fn update_buff_time_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BuffTime), With<Buff>>,
    time: Res<Time>,
) {
    for (entity, mut buff_time) in query.iter_mut() {
        if let Some(looper_timer) = &mut buff_time.looper_timer {
            looper_timer.tick(time.delta());
            if looper_timer.finished() {
                commands.trigger_targets(
                    EffectGraphExecEvent {
                        entry_exec_pin: EffectNodeBuffEntry::OUTPUT_EXEC_LOOPER.into(),
                        execute_in_graph_state: Some(EffectGraphState::Active),
                        slot_value_map: None,
                    },
                    entity,
                );
            }
        }

        buff_time.once_timer.tick(time.delta());
        if buff_time.once_timer.finished() {
            commands.trigger_targets(
                EffectGraphExecEvent {
                    entry_exec_pin: EffectNodeBuffEntry::OUTPUT_EXEC_END.into(),
                    execute_in_graph_state: Some(EffectGraphState::Active),
                    slot_value_map: None,
                },
                entity,
            );
        }
    }
}
