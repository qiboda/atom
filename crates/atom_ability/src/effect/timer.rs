//! Effect 计时：记录持续时间并在到时触发图 END 执行。

use bevy::{
    prelude::{Commands, Component, Entity, Query, Res},
    reflect::Reflect,
    time::Time,
};

use crate::graph::{event::EffectGraphExecEvent, state::EffectGraphState};

use super::node::effect_entry::EffectNodeEffectEntry;

/// Effect 计时组件：已过时间与总时长。
#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct EffectTime {
    /// 已过时间（秒）。
    pub elapse: f32,
    /// 总时长（秒）。
    pub duration: f32,
}

/// 推进效果计时。
pub fn update_effect_timer_system(mut query: Query<&mut EffectTime>, time: Res<Time>) {
    for mut effect in query.iter_mut() {
        effect.elapse += time.delta_secs();
    }
}

/// 到时触发图 END 执行：已过时间达到总时长时，向 effect 图实例触发 `end` 执行口
/// （不直接 despawn——图实例是效果的子实体，图 END 链完成后由效果移除流程清理）。
pub fn time_end_destroy_effect(mut commands: Commands, query: Query<(Entity, &EffectTime)>) {
    for (entity, effect) in query.iter() {
        if effect.elapse >= effect.duration {
            commands.trigger(EffectGraphExecEvent {
                entry_exec_pin: EffectNodeEffectEntry::OUTPUT_EXEC_END.into(),
                execute_in_graph_state: Some(EffectGraphState::Active),
                slot_value_map: None,
                ability_entity: entity,
            });
        }
    }
}
