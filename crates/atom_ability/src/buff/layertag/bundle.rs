//! Buff 状态层标签组件包：从数据表原始标签构建 buff 各阶段标签组件。

use atom_datatables::effect::RevertableLayerTag;
use atom_layertag::container_op::LayerTagContainer;
use bevy::{
    log::warn,
    prelude::{Bundle, Res},
    reflect::Reflect,
};

use crate::stateset::StateLayerTagRegistry;

use super::tag::{
    BuffAbortDisableLayerTagContainer, BuffAbortRequiredLayerTagContainer,
    BuffAddedLayerTagContainer, BuffRemovedLayerTagContainer, BuffStartDisableLayerTagContainer,
    BuffStartRequiredLayerTagContainer,
};

/// Buff 开始阶段标签组件包：所需/禁用/添加/移除四类标签。
#[derive(Debug, Default, Bundle, Reflect)]
pub struct BuffStartTagBundle {
    /// 开始所需标签。
    pub required_layertags: BuffStartRequiredLayerTagContainer,
    /// 开始禁用标签。
    pub disable_layertags: BuffStartDisableLayerTagContainer,
    /// 开始时要添加的标签。
    pub added_layertags: BuffAddedLayerTagContainer,
    /// 开始时要移除的标签。
    pub removed_layertags: BuffRemovedLayerTagContainer,
}

impl BuffStartTagBundle {
    /// 从数据表原始标签字符串构造组件包（未注册的标签记录 warning 并跳过）。
    pub fn new(
        required_layertags: &[String],
        disable_layertags: &[String],
        added_layertags: &[RevertableLayerTag],
        removed_layertags: &[RevertableLayerTag],
        state_registry: &Res<StateLayerTagRegistry>,
    ) -> Self {
        let mut bundle = BuffStartTagBundle::default();

        for raw_layertag in required_layertags.iter() {
            match state_registry.0.request_from_raw(raw_layertag) {
                Some(layertag) => {
                    bundle.required_layertags.0.add_layertag(layertag);
                }
                None => {
                    warn!("layertag not found registry: {}", raw_layertag)
                }
            }
        }

        for raw_layertag in disable_layertags.iter() {
            match state_registry.0.request_from_raw(raw_layertag) {
                Some(layertag) => {
                    bundle.disable_layertags.0.add_layertag(layertag);
                }
                None => {
                    warn!("layertag not found registry: {}", raw_layertag)
                }
            }
        }

        for revertable_layertag in added_layertags.iter() {
            match state_registry
                .0
                .request_from_raw(&revertable_layertag.raw_layertag)
            {
                Some(layertag) => {
                    bundle
                        .added_layertags
                        .layer_tag_container
                        .add_layertag(layertag);
                    bundle.added_layertags.revert = revertable_layertag.revertable.into();
                }
                None => {
                    warn!("layertag not found registry: {:?}", revertable_layertag)
                }
            }
        }

        for revertable_layertag in removed_layertags.iter() {
            match state_registry
                .0
                .request_from_raw(revertable_layertag.raw_layertag.as_str())
            {
                Some(layertag) => {
                    bundle
                        .removed_layertags
                        .layer_tag_container
                        .add_layertag(layertag);
                    bundle.removed_layertags.revert = revertable_layertag.revertable.into();
                }
                None => {
                    warn!("layertag not found registry: {:?}", revertable_layertag)
                }
            }
        }

        bundle
    }
}

/// Buff 中断阶段标签组件包：所需/禁用两类标签。
#[derive(Debug, Default, Bundle, Reflect)]
pub struct BuffAbortTagBundle {
    /// 中断所需标签。
    pub required_layer_tag: BuffAbortRequiredLayerTagContainer,
    /// 中断禁用标签。
    pub disable_layer_tag: BuffAbortDisableLayerTagContainer,
}

impl BuffAbortTagBundle {
    /// 从数据表原始标签字符串构造组件包（未注册的标签记录 warning 并跳过）。
    pub fn new(
        required_layertags: &[String],
        disable_layertags: &[String],
        state_registry: &Res<StateLayerTagRegistry>,
    ) -> Self {
        let mut bundle = BuffAbortTagBundle::default();

        for raw_layertag in required_layertags.iter() {
            match state_registry.0.request_from_raw(raw_layertag) {
                Some(layertag) => {
                    bundle.required_layer_tag.0.add_layertag(layertag);
                }
                None => {
                    warn!("layertag not found registry: {}", raw_layertag)
                }
            }
        }

        for raw_layertag in disable_layertags.iter() {
            match state_registry.0.request_from_raw(raw_layertag) {
                Some(layertag) => {
                    bundle.disable_layer_tag.0.add_layertag(layertag);
                }
                None => {
                    warn!("layertag not found registry: {}", raw_layertag)
                }
            }
        }

        bundle
    }
}
