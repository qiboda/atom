//! Effect Graph 状态管理：图级执行状态与销毁清理。

use bevy::log::info;
use bevy::prelude::*;

use super::{context::EffectGraphContext, node::EffectNodeExecuteState};

/// Effect Graph 的节流状态：是否暂停执行。
#[derive(Debug, Component, Default, Copy, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum EffectGraphTickState {
    /// 正常运行（默认）。
    #[default]
    Ticked,
    /// 暂停执行。
    Paused,
}

/// Effect Graph 的生命周期状态。
#[derive(Debug, Component, Default, PartialEq, Eq, Hash, Reflect, Clone, Copy)]
#[reflect(Component)]
pub enum EffectGraphState {
    /// 未激活（默认）。
    #[default]
    Inactive,
    /// 激活执行中。
    Active,
    /// 待销毁：等待所有节点空闲后移除图实体。
    ToRemove,
}

/// 重置图状态：当所有状态节点处于 `Idle` 时，将 `Active` 的图置回 `Inactive`。
pub fn reset_effect_graph_state(
    mut query: Query<(&mut EffectGraphState, &EffectGraphContext)>,
    node_state_query: Query<&EffectNodeExecuteState>,
) {
    for (mut state, context) in query.iter_mut() {
        match *state {
            EffectGraphState::Inactive => {}
            EffectGraphState::Active => {
                if context.state_nodes.iter().all(|node| {
                    if let Ok(node_state) = node_state_query.get(*node)
                        && *node_state == EffectNodeExecuteState::Idle
                    {
                        return true;
                    }
                    false
                }) {
                    *state = EffectGraphState::Inactive;
                }
            }
            EffectGraphState::ToRemove => {}
        }
    }
}

/// 销毁待移除的图：对 `ToRemove` 且所有状态节点均空闲的图，despawn 其实体。
pub fn update_to_despawn_effect_graph(
    mut commands: Commands,
    query: Query<(Entity, &EffectGraphState, &EffectGraphContext)>,
    node_state_query: Query<&EffectNodeExecuteState>,
) {
    for (graph_entity, state, context) in query.iter() {
        if *state == EffectGraphState::ToRemove
            && context.state_nodes.iter().all(|node| {
                if let Ok(node_state) = node_state_query.get(*node)
                    && *node_state == EffectNodeExecuteState::Idle
                {
                    return true;
                }
                false
            })
        {
            commands.entity(graph_entity).despawn();
            info!("despawn graph: {:?}", graph_entity);
        }
    }
}
