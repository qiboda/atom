pub mod bundle;
pub mod event;
pub mod graph_map;
pub mod state;
pub mod tag;
pub mod timer;

use bevy::prelude::{App, IntoSystemConfigs, Last, Plugin, PostUpdate, PreUpdate, Update};

use self::{
    event::{
        receive_start_effect, EffectAbortEvent, EffectPauseEvent, EffectResumeEvent,
        EffectStartEvent,
    },
    graph_map::EffectGraphMap,
    state::{on_remove_effect, update_to_active_state, update_to_unactive_state},
    tag::{
        effect_tag_revert_apply_system, effect_tag_start_apply_system,
        effect_tag_start_check_system,
    },
    timer::{time_end_destroy_effect, update_effect_timer_system},
};

pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(EffectGraphMap::default())
            .add_event::<EffectStartEvent>()
            .add_event::<EffectAbortEvent>()
            .add_event::<EffectPauseEvent>()
            .add_event::<EffectResumeEvent>()
            .add_systems(
                PreUpdate,
                (
                    receive_start_effect,
                    effect_tag_start_check_system,
                    update_to_active_state,
                    effect_tag_start_apply_system,
                    update_to_active_state,
                )
                    .chain(),
            )
            .add_systems(Update, update_effect_timer_system)
            .add_systems(
                PostUpdate,
                (
                    update_to_unactive_state,
                    effect_tag_revert_apply_system,
                    update_to_unactive_state,
                )
                    .chain(),
            )
            .add_systems(Last, on_remove_effect)
            .add_systems(Last, time_end_destroy_effect);
    }
}
