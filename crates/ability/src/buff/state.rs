use bevy::prelude::*;

use crate::graph::state::{EffectGraphState, EffectGraphTickState};

#[derive(Debug, Component, Default, Reflect, Copy, Clone)]
pub struct Buff;

#[derive(Debug, Component, Default, Reflect, PartialEq, Eq, Copy, Clone)]
pub enum BuffExecuteState {
    #[default]
    Inactive,
    Active,
    ToRemove,
}

#[derive(Debug, Component, Default, Reflect, PartialEq, Eq, Copy, Clone)]
pub enum BuffTickState {
    #[default]
    Ticked,
    Paused,
}

/// 根据子图的状态更新技能的状态。
/// 如果有至少一个子图正在执行，那么这个技能就是执行中的。
/// 如果有至少一个子图Idle那么这个技能就是Idle的。
pub fn update_buff_state(
    mut query: Query<(&Children, &mut BuffExecuteState), With<Buff>>,
    graph_query: Query<&EffectGraphState>,
) {
    for (children, mut state) in query.iter_mut() {
        let mut any_active = false;
        let mut any_inactive = false;
        for child in children.iter() {
            if let Ok(graph_state) = graph_query.get(*child) {
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

/// 根据子图的状态更新技能的状态。
/// 如果所有的子图都ToRemove，那么这个技能就是待移除的。
/// 如果所有的子图全部都pause，则这个技能是pause的。
pub fn update_buff_tick_state(
    mut query: Query<(&Children, &mut BuffTickState), With<Buff>>,
    graph_query: Query<&EffectGraphTickState>,
) {
    for (children, mut state) in query.iter_mut() {
        let mut any_ticked = true;
        for child in children.iter() {
            if let Ok(graph_state) = graph_query.get(*child) {
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
