//! Effect（效果）模块：技能/增益的图实例实体——状态机、状态层标签、计时与销毁。

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
    state::{on_remove_effect, update_to_active_state, update_to_inactive_state},
    tag::{
        effect_tag_revert_apply_system, effect_tag_start_apply_system,
        effect_tag_start_check_system,
    },
    timer::{time_end_destroy_effect, update_effect_timer_system},
};

/// Effect 子系统插件：注册效果事件、状态更新、标签应用与计时销毁系统。
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
                    update_to_inactive_state,
                    effect_tag_revert_apply_system,
                    update_to_inactive_state,
                )
                    .chain(),
            )
            .add_systems(Last, on_remove_effect)
            .add_systems(Last, time_end_destroy_effect);
    }
}
