//! Buff 计时：一次时长计时 + 可选循环计时。

use bevy::{
    prelude::{Commands, Component, Entity, Query, Res, With},
    reflect::Reflect,
    time::{Time, Timer, TimerMode},
};

use crate::graph::{event::EffectGraphExecEvent, state::EffectGraphState};

use super::{node::buff_entry::EffectNodeBuffEntry, state::Buff};

/// Buff 计时器组件：一次计时（结束触发 end）+ 可选循环计时（每周期触发 looper）。
#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct BuffTime {
    /// 一次计时器：到时触发 end。
    pub once_timer: Timer,
    /// 循环计时器：每周期触发 looper。
    pub looper_timer: Option<Timer>,
}

impl BuffTime {
    /// 创建计时器：`once_duration` 为总时长，`looper_duration` 为循环周期（`None` 无循环）。
    pub fn new(once_duration: f32, looper_duration: Option<f32>) -> Self {
        Self {
            once_timer: Timer::from_seconds(once_duration, TimerMode::Once),
            looper_timer: looper_duration.map(|x| Timer::from_seconds(x, TimerMode::Repeating)),
        }
    }
}

/// 驱动 buff 计时：循环到时触发图 LOOPER 执行，总时长到时触发图 END 执行。
pub fn update_buff_time_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut BuffTime), With<Buff>>,
    time: Res<Time>,
) {
    for (entity, mut buff_time) in query.iter_mut() {
        if let Some(looper_timer) = &mut buff_time.looper_timer {
            looper_timer.tick(time.delta());
            if looper_timer.is_finished() {
                commands.trigger(EffectGraphExecEvent {
                    entry_exec_pin: EffectNodeBuffEntry::OUTPUT_EXEC_LOOPER.into(),
                    execute_in_graph_state: Some(EffectGraphState::Active),
                    slot_value_map: None,
                    ability_entity: entity,
                });
            }
        }

        buff_time.once_timer.tick(time.delta());
        if buff_time.once_timer.is_finished() {
            commands.trigger(EffectGraphExecEvent {
                entry_exec_pin: EffectNodeBuffEntry::OUTPUT_EXEC_END.into(),
                execute_in_graph_state: Some(EffectGraphState::Active),
                slot_value_map: None,
                ability_entity: entity,
            });
        }
    }
}
