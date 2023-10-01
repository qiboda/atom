pub mod bundle;
pub mod timer;

use bevy::{
    prelude::{App, Component, Last, Plugin, Update},
    reflect::Reflect,
};

use self::timer::{
    destroy_ability_effect, update_ability_effect_delay_timer, update_ability_effect_end_timer,
    update_ability_effect_loop_timer, update_ability_effect_start_timer,
    update_ability_effect_timer_system,
};

pub struct AbilityEffectPlugin;

impl Plugin for AbilityEffectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_ability_effect_timer_system,
                update_ability_effect_start_timer,
                update_ability_effect_delay_timer,
                update_ability_effect_loop_timer,
                update_ability_effect_end_timer,
            ), // .chain(),
        )
        .add_systems(Last, destroy_ability_effect);
    }
}

#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct AbilityEffect {
    pub elapse: f32,
    pub duration: f32,
}
