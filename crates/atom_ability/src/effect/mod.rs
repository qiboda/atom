//! Effect（效果）模块：技能/增益的图实例实体——状态机、状态层标签、计时与图执行入口。

pub mod bundle;
pub mod event;
pub mod node;
pub mod state;
pub mod tag;
pub mod timer;

use bevy::prelude::*;

use crate::graph::EffectGraphUpdateSystems;

use self::{
    event::trigger_effect_start,
    node::effect_entry::EffectNodeEffectEntryPlugin,
    state::{on_remove_effect, update_to_active_state, update_to_inactive_state},
    tag::{
        effect_tag_revert_apply_system, effect_tag_start_apply_system,
        effect_tag_start_check_system,
    },
    timer::{time_end_destroy_effect, update_effect_timer_system},
};

/// Effect 子系统插件：注册效果事件 observer、状态更新、标签应用与计时系统。
///
/// 图交互（start/end 执行、图实例挂载/清理）通过 [`crate::graph::EffectGraphPlugin`]
/// 的 observer 基础设施完成——effect 实体需挂 [`crate::graph::EffectGraphOwner`] 标记
/// 以拥有图实例（E2：graph_class 由生成效果实体的 bundle 数据继承，或经
/// [`crate::graph::event::EffectGraphAddEvent`] 传入）。
#[derive(Debug, Default)]
pub struct EffectPlugin;

impl Plugin for EffectPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                EffectUpdateSystems::UpdateTime,
                EffectUpdateSystems::UpdateState,
            )
                .chain(),
        )
        .add_observer(trigger_effect_start)
        .add_systems(
            Update,
            (
                effect_tag_start_check_system,
                update_to_active_state,
                effect_tag_start_apply_system,
            )
                .after(EffectGraphUpdateSystems::UpdateState)
                .in_set(EffectUpdateSystems::UpdateState),
        )
        .add_systems(
            Update,
            update_effect_timer_system.in_set(EffectUpdateSystems::UpdateTime),
        )
        .add_systems(
            PostUpdate,
            (update_to_inactive_state, effect_tag_revert_apply_system).chain(),
        )
        .add_systems(Last, on_remove_effect)
        .add_systems(Last, time_end_destroy_effect)
        .add_plugins(EffectNodeEffectEntryPlugin);
    }
}

/// Effect 更新调度集：计时（UpdateTime）先于状态更新（UpdateState）。
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EffectUpdateSystems {
    /// 计时更新。
    UpdateTime,
    /// 状态更新。
    UpdateState,
}
