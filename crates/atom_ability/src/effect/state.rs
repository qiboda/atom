//! Effect 状态机：Inactive → CheckCanActive → ActiveBefore → Active → BeforeInactive。

use bevy::prelude::*;

use crate::graph::{
    event::{EffectGraphExecEvent, EffectGraphRemoveEvent},
    state::EffectGraphState,
};

use super::node::effect_entry::EffectNodeEffectEntry;

/// add ability to entity
/// active ability
/// inactive ability
/// receive input
/// Effect 状态机：`Inactive` 非激活、`CheckCanActive` 检查开始条件、
/// `ActiveBefore` 激活前（应用标签）、`Active` 激活、`BeforeInactive` 失活前（回滚标签）。
#[derive(Debug, Component, Default, Reflect, Copy, Clone, PartialEq)]
pub enum EffectState {
    /// 非激活（默认）。
    #[default]
    Inactive,
    /// 检查是否满足激活条件。
    CheckCanActive,
    /// 激活前（应用状态层标签）。
    ActiveBefore,
    /// 激活。
    Active,
    /// 失活前（回滚状态层标签）。
    BeforeInactive,
}

/// set active from ability start, so set inactive when all children finished.
/// 推进状态收尾：Active → BeforeInactive → Inactive（子效果全部结束时回到非激活）。
pub fn update_to_inactive_state(mut effect_query: Query<&mut EffectState>) {
    for mut effect_state in effect_query.iter_mut() {
        match *effect_state {
            EffectState::Inactive => {}
            EffectState::CheckCanActive => {}
            EffectState::ActiveBefore => {}
            EffectState::Active => {
                *effect_state = EffectState::BeforeInactive;
            }
            EffectState::BeforeInactive => {
                *effect_state = EffectState::Inactive;
            }
        }
    }
}

/// 推进激活流程：CheckCanActive → ActiveBefore → Active，激活时向图入口节点触发 start 执行。
///
/// 图实例是效果的子实体（经 [`crate::graph::event::EffectGraphAddEvent`] 挂载），
/// 此处通过 [`EffectGraphExecEvent`] 以效果实体为事件目标触发图 start 执行。
pub fn update_to_active_state(
    mut commands: Commands,
    mut state_query: Query<(Entity, &mut EffectState)>,
) {
    for (entity, mut state) in state_query.iter_mut() {
        match *state {
            EffectState::Inactive => {}
            EffectState::CheckCanActive => {
                *state = EffectState::ActiveBefore;
            }
            EffectState::ActiveBefore => {
                *state = EffectState::Active;

                commands.trigger(EffectGraphExecEvent {
                    entry_exec_pin: EffectNodeEffectEntry::OUTPUT_EXEC_START.into(),
                    execute_in_graph_state: Some(EffectGraphState::Inactive),
                    slot_value_map: None,
                    ability_entity: entity,
                });
            }
            EffectState::Active => {}
            EffectState::BeforeInactive => {}
        }
    }
}

/// 效果移除时清理其图实例。
///
/// Bevy 0.19 的 `despawn()` 会递归 despawn 子实体，图实例作为效果的子实体随效果一起被清理；
/// 此处额外触发 [`EffectGraphRemoveEvent`] 兜底——若 `EffectState` 被单独移除而实体仍在，
/// 将存活的图实例标记为 `ToRemove` 走优雅退出流程。
pub fn on_remove_effect(
    mut removed_effect: RemovedComponents<EffectState>,
    mut commands: Commands,
) {
    for effect in removed_effect.read() {
        commands.trigger(EffectGraphRemoveEvent {
            ability_entity: effect,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::MinimalPlugins;

    #[test]
    fn update_to_active_state_progresses_check_to_active() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(EffectState::CheckCanActive).id();
        app.add_systems(Update, update_to_active_state);

        app.update();
        assert_eq!(
            *app.world()
                .entity(entity)
                .get::<EffectState>()
                .expect("effect state must exist"),
            EffectState::ActiveBefore
        );

        app.update();
        assert_eq!(
            *app.world()
                .entity(entity)
                .get::<EffectState>()
                .expect("effect state must exist"),
            EffectState::Active
        );
    }

    #[test]
    fn update_to_inactive_state_progresses_to_inactive() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let entity = app.world_mut().spawn(EffectState::Active).id();
        app.add_systems(Update, update_to_inactive_state);

        app.update();
        assert_eq!(
            *app.world()
                .entity(entity)
                .get::<EffectState>()
                .expect("effect state must exist"),
            EffectState::BeforeInactive
        );

        app.update();
        assert_eq!(
            *app.world()
                .entity(entity)
                .get::<EffectState>()
                .expect("effect state must exist"),
            EffectState::Inactive
        );
    }
}
