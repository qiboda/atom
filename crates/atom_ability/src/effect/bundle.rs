//! Effect 状态层标签组件包。

use crate::effect::tag::{
    EffectAbortDisableLayerTagContainer, EffectAbortRequiredLayerTagContainer,
    EffectAddedLayerTagContainer, EffectRemovedLayerTagContainer,
    EffectStartDisableLayerTagContainer, EffectStartRequiredLayerTagContainer,
};
use bevy::{prelude::Bundle, reflect::Reflect};

/// Effect 开始阶段标签组件包：所需/禁用/添加/移除四类标签。
#[derive(Debug, Default, Bundle, Reflect)]
pub struct EffectStartTagBundle {
    /// 开始所需标签。
    pub required_layer_tag: EffectStartRequiredLayerTagContainer,
    /// 开始禁用标签。
    pub disable_layer_tag: EffectStartDisableLayerTagContainer,
    /// 开始时要添加的标签。
    pub added_layer_tag: EffectAddedLayerTagContainer,
    /// 开始时要移除的标签。
    pub removed_layer_tag: EffectRemovedLayerTagContainer,
}

/// Effect 中断阶段标签组件包：所需/禁用两类标签。
#[derive(Debug, Default, Bundle, Reflect)]
pub struct EffectAbortTagBundle {
    /// 中断所需标签。
    pub required_layer_tag: EffectAbortRequiredLayerTagContainer,
    /// 中断禁用标签。
    pub disable_layer_tag: EffectAbortDisableLayerTagContainer,
}
