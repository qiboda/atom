//! Effect 状态层标签：效果各阶段所需/禁用/增删的标签与相关系统。

use std::ops::Not;

use atom_layertag::{
    container_op::{
        LayerTagContainerConditionRequired, LayerTagContainerConditionWithout,
        LayerTagContainerOpAdd, LayerTagContainerOpRemove,
    },
    count_container::CountLayerTagContainer,
};
use bevy::prelude::*;

use crate::stateset::StateLayerTagContainer;

use super::state::EffectState;

/// Effect 开始所需的状态层标签容器（全部满足才可开始）。
#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct EffectStartRequiredLayerTagContainer(pub CountLayerTagContainer);

/// Effect 开始禁用的状态层标签容器（存在任一即不可开始）。
#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct EffectStartDisableLayerTagContainer(pub CountLayerTagContainer);

/// Effect 中断所需的状态层标签容器（全部满足才可中断）。
#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct EffectAbortRequiredLayerTagContainer(pub CountLayerTagContainer);

/// Effect 中断禁用的状态层标签容器（存在任一即不可中断）。
#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct EffectAbortDisableLayerTagContainer(pub CountLayerTagContainer);

/// Effect 结束后是否回滚状态层标签。
#[derive(Debug, Default, Reflect, PartialEq, Eq, Clone)]
pub enum EffectLayerTagContainerRevert {
    /// 不回滚（默认）。
    #[default]
    No,
    /// 回滚。
    Yes,
}

impl From<bool> for EffectLayerTagContainerRevert {
    fn from(value: bool) -> Self {
        if value {
            EffectLayerTagContainerRevert::Yes
        } else {
            EffectLayerTagContainerRevert::No
        }
    }
}

/// Effect 开始时要添加的状态层标签集合（带回滚标记）。
#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct EffectAddedLayerTagContainer {
    /// 要添加的标签集合。
    pub layer_tag_container: CountLayerTagContainer,
    /// 结束后是否回滚。
    pub revert: EffectLayerTagContainerRevert,
}

/// Effect 开始时要移除的状态层标签集合（带回滚标记）。
#[derive(Component, Debug, Default, Reflect, Clone)]
pub struct EffectRemovedLayerTagContainer {
    /// 要移除的标签集合。
    pub layer_tag_container: CountLayerTagContainer,
    /// 结束后是否回滚。
    pub revert: EffectLayerTagContainerRevert,
}

/// 检查待激活效果的开始条件：不满足所需/禁用标签则置回非激活。
pub fn effect_tag_start_check_system(
    state_set_query: Query<&StateLayerTagContainer>,
    mut query: Query<(
        &ChildOf,
        &mut EffectState,
        &EffectStartRequiredLayerTagContainer,
        &EffectStartDisableLayerTagContainer,
    )>,
) {
    for (parent, mut effect_state, required_tag, disable_tag) in query.iter_mut() {
        if *effect_state == EffectState::CheckCanActive {
            let state_layer_tag_container = state_set_query
                .get(parent.parent())
                .expect("state layer tag container must exist on parent");

            let can_start = state_layer_tag_container
                .0
                .condition(LayerTagContainerConditionRequired, &required_tag.0)
                && state_layer_tag_container
                    .0
                    .condition(LayerTagContainerConditionWithout, &disable_tag.0);
            if can_start.not() {
                *effect_state = EffectState::Inactive;
            }
        }
    }
}

/// 对刚激活的效果应用状态层标签增删。
pub fn effect_tag_start_apply_system(
    mut state_set_query: Query<&mut StateLayerTagContainer>,
    query: Query<(
        &ChildOf,
        &EffectState,
        &EffectAddedLayerTagContainer,
        &EffectRemovedLayerTagContainer,
    )>,
) {
    for (parent, effect_state, added_tag, removed_tag) in query.iter() {
        if *effect_state == EffectState::ActiveBefore {
            let mut state_layer_tag_container = state_set_query
                .get_mut(parent.parent())
                .expect("state layer tag container must exist on parent");

            state_layer_tag_container
                .0
                .receive_op(LayerTagContainerOpAdd, &added_tag.layer_tag_container);

            state_layer_tag_container
                .0
                .receive_op(LayerTagContainerOpRemove, &removed_tag.layer_tag_container);
        }
    }
}

/// 对即将失活的效果回滚其标记为可回滚的标签增删。
pub fn effect_tag_revert_apply_system(
    mut state_set_query: Query<&mut StateLayerTagContainer>,
    query: Query<(
        &ChildOf,
        &EffectState,
        &EffectAddedLayerTagContainer,
        &EffectRemovedLayerTagContainer,
    )>,
) {
    for (parent, effect_state, added_tag, removed_tag) in query.iter() {
        if *effect_state == EffectState::BeforeInactive {
            let mut state_layer_tag_container = state_set_query
                .get_mut(parent.parent())
                .expect("state layer tag container must exist on parent");

            if added_tag.revert == EffectLayerTagContainerRevert::Yes {
                state_layer_tag_container
                    .0
                    .receive_op(LayerTagContainerOpRemove, &added_tag.layer_tag_container);
            }

            if removed_tag.revert == EffectLayerTagContainerRevert::Yes {
                state_layer_tag_container
                    .0
                    .receive_op(LayerTagContainerOpAdd, &removed_tag.layer_tag_container);
            }
        }
    }
}
