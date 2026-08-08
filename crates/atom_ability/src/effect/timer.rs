//! Effect 计时：记录持续时间并在到时销毁效果实体。

use bevy::{
    prelude::{Commands, Component, Entity, Query, Res},
    reflect::Reflect,
    time::Time,
};

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
        effect.elapse += time.delta_seconds();
    }
}

/// 到时销毁：已过时间达到总时长时 despawn 效果实体。
pub fn time_end_destroy_effect(mut commands: Commands, query: Query<(Entity, &EffectTime)>) {
    for (entity, effect) in query.iter() {
        if effect.elapse >= effect.duration {
            commands.entity(entity).despawn();
        }
    }
}
