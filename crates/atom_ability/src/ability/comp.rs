//! 技能组件：技能标记、执行状态与节流状态。

use bevy::prelude::*;

use crate::graph::state::{EffectGraphState, EffectGraphTickState};

// only ability with this component
/// 技能标记组件：只有带此组件的实体才是技能实体。
#[derive(Debug, Component, Default, Reflect, Copy, Clone)]
pub struct Ability;

/// 技能执行状态。
#[derive(Debug, Component, Default, PartialEq, Eq, Reflect, Copy, Clone)]
pub enum AbilityExecuteState {
    /// 未激活（默认）。
    #[default]
    Inactive,
    /// 执行中。
    Active,
    /// 待销毁（所有子图已移除）。
    ToRemove,
}

/// 技能节流状态。
#[derive(Debug, Component, Default, PartialEq, Eq, Reflect, Copy, Clone)]
pub enum AbilityTickState {
    /// 正常运行（默认）。
    #[default]
    Ticked,
    /// 暂停。
    Paused,
}

/// 技能数据占位组件。
#[derive(Debug, Component, Default, Reflect)]
pub struct AbilityData;

/// 根据子图的状态更新技能的状态。
/// 如果有至少一个子图正在执行，那么这个技能就是执行中的。
/// 如果有至少一个子图Idle那么这个技能就是Idle的。
pub fn update_ability_state(
    mut query: Query<(&Children, &mut AbilityExecuteState), With<Ability>>,
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
            *state = AbilityExecuteState::Active;
        } else if any_inactive {
            *state = AbilityExecuteState::Inactive;
        } else {
            *state = AbilityExecuteState::ToRemove;
        }
    }
}

/// 根据子图的状态更新技能的状态。
/// 如果所有的子图都ToRemove，那么这个技能就是待移除的。
/// 如果所有的子图全部都pause，则这个技能是pause的。
pub fn update_ability_tick_state(
    mut query: Query<(&Children, &mut AbilityTickState), With<Ability>>,
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
            *state = AbilityTickState::Ticked;
        } else {
            *state = AbilityTickState::Paused;
        }
    }
}
