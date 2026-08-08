//! Effect 状态机：Inactive → CheckCanActive → ActiveBefore → Active → BeforeInactive。

use bevy::prelude::*;

use crate::graph::{
    context::EffectGraphContext,
    event::{EffectNodeEvent, EffectNodeStartEvent},
    state::EffectGraphState,
};

use super::graph_map::EffectGraphMap;

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

/// 推进激活流程：CheckCanActive → ActiveBefore → Active，激活时向图的入口节点发送开始事件。
pub fn update_to_active_state(
    mut state_query: Query<(Entity, &mut EffectState)>,
    graph_query: Query<&EffectGraphContext>,
    effect_graph_map: Res<EffectGraphMap>,
    mut event_writer: EventWriter<EffectNodeStartEvent>,
) {
    for (entity, mut state) in state_query.iter_mut() {
        match *state {
            EffectState::Inactive => {}
            EffectState::CheckCanActive => {
                *state = EffectState::ActiveBefore;
            }
            EffectState::ActiveBefore => {
                *state = EffectState::Active;

                let graph = effect_graph_map
                    .map
                    .get(&entity)
                    .expect("effect graph must exist in map");
                let graph_context = graph_query
                    .get(graph.get_entity())
                    .expect("effect graph context must exist");
                if let Some(entry_node) = graph_context.entry_node {
                    event_writer.send(EffectNodeStartEvent::new(entry_node));
                }
            }
            EffectState::Active => {}
            EffectState::BeforeInactive => {}
        }
    }
}

/// 效果移除时：将效果关联的图标记为待销毁。
pub fn on_remove_effect(
    mut removed_ability: RemovedComponents<EffectState>,
    mut effect_graph_map: ResMut<EffectGraphMap>,
    mut query: Query<&mut EffectGraphState>,
) {
    for ability in removed_ability.read() {
        if let Some(graph_ref) = effect_graph_map.map.remove(&ability) {
            let mut graph_state = query
                    .get_mut(graph_ref.get_entity())
                    .expect("effect graph state must exist");
            *graph_state = EffectGraphState::ToRemove;
        }
    }
}
