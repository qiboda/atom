use bevy::{
    prelude::{Commands, Component, Entity, Query, Res},
    reflect::Reflect,
    time::Time,
};

#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct EffectTime {
    pub elapse: f32,
    pub duration: f32,
}

pub fn update_effect_timer_system(mut query: Query<&mut EffectTime>, time: Res<Time>) {
    for mut effect in query.iter_mut() {
        effect.elapse += time.delta_seconds();
    }
}

pub fn time_end_destroy_effect(mut commands: Commands, query: Query<(Entity, &EffectTime)>) {
    for (entity, effect) in query.iter() {
        if effect.elapse >= effect.duration {
            commands.entity(entity).despawn();
        }
    }
}
