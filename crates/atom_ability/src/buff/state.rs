//! Buff 状态：标记、执行状态与节流状态。

use bevy::prelude::*;

use crate::graph::state::{EffectGraphState, EffectGraphTickState};

/// Buff 标记组件。
#[derive(Debug, Component, Default, Reflect, Copy, Clone)]
pub struct Buff;

/// Buff 执行状态。
#[derive(Debug, Component, Default, Reflect, PartialEq, Eq, Copy, Clone)]
pub enum BuffExecuteState {
    /// 未激活（默认）。
    #[default]
    Inactive,
    /// 执行中。
    Active,
    /// 待销毁。
    ToRemove,
}

/// Buff 节流状态。
#[derive(Debug, Component, Default, Reflect, PartialEq, Eq, Copy, Clone)]
pub enum BuffTickState {
    /// 正常运行（默认）。
    #[default]
    Ticked,
    /// 暂停。
    Paused,
}

/// 根据子图的状态更新 buff 的状态。
/// 如果有至少一个子图正在执行，那么这个 buff 就是执行中的。
/// 如果有至少一个子图 Idle 那么这个 buff 就是 Idle 的。
pub fn update_buff_state(
    mut query: Query<(&Children, &mut BuffExecuteState), With<Buff>>,
    graph_query: Query<&EffectGraphState>,
) {
    for (children, mut state) in query.iter_mut() {
        let mut any_active = false;
        let mut any_inactive = false;
        for child in children.iter() {
            if let Ok(graph_state) = graph_query.get(child) {
                match graph_state {
                    EffectGraphState::Inactive => {
                        any_inactive = true;
                    }
                    EffectGraphState::Active => {
                        any_active = true;
                    }
                    EffectGraphState::ToRemove => {}
                }
            }
        }

        if any_active {
            *state = BuffExecuteState::Active;
        } else if any_inactive {
            *state = BuffExecuteState::Inactive;
        } else {
            *state = BuffExecuteState::ToRemove;
        }
    }
}

/// 根据子图的状态更新 buff 的状态。
/// 如果所有的子图都 ToRemove，那么这个 buff 就是待移除的。
/// 如果所有的子图全部都 pause，则这个 buff 是 pause 的。
pub fn update_buff_tick_state(
    mut query: Query<(&Children, &mut BuffTickState), With<Buff>>,
    graph_query: Query<&EffectGraphTickState>,
) {
    for (children, mut state) in query.iter_mut() {
        let mut any_ticked = true;
        for child in children.iter() {
            if let Ok(graph_state) = graph_query.get(child) {
                match graph_state {
                    EffectGraphTickState::Ticked => {
                        any_ticked = true;
                    }
                    EffectGraphTickState::Paused => {}
                }
            }
        }

        if any_ticked {
            *state = BuffTickState::Ticked;
        } else {
            *state = BuffTickState::Paused;
        }
    }
}
