//! Buff 状态层标签构建：从数据表原始标签构建 buff 各阶段标签组件实例。

use atom_layertag::container_op::LayerTagContainer;
use bevy::{log::warn, prelude::Res};

use crate::{config::RevertableLayerTag, stateset::StateLayerTagRegistry};

use super::tag::{
    BuffAbortDisableLayerTagContainer, BuffAbortRequiredLayerTagContainer,
    BuffAddedLayerTagContainer, BuffRemovedLayerTagContainer, BuffStartDisableLayerTagContainer,
    BuffStartRequiredLayerTagContainer,
};

/// 从数据表原始标签构建 buff 开始阶段四个标签容器
/// （所需/禁用/添加/移除；未注册的标签记录 warning 并跳过）。
pub fn build_buff_start_tags(
    required_layertags: &[String],
    disable_layertags: &[String],
    added_layertags: &[RevertableLayerTag],
    removed_layertags: &[RevertableLayerTag],
    state_registry: &Res<StateLayerTagRegistry>,
) -> (
    BuffStartRequiredLayerTagContainer,
    BuffStartDisableLayerTagContainer,
    BuffAddedLayerTagContainer,
    BuffRemovedLayerTagContainer,
) {
    let mut required = BuffStartRequiredLayerTagContainer::default();
    let mut disable = BuffStartDisableLayerTagContainer::default();
    let mut added = BuffAddedLayerTagContainer::default();
    let mut removed = BuffRemovedLayerTagContainer::default();

    for raw_layertag in required_layertags.iter() {
        match state_registry.0.request_from_raw(raw_layertag) {
            Some(layertag) => {
                required.0.add_layertag(layertag);
            }
            None => {
                warn!("layertag not found registry: {}", raw_layertag)
            }
        }
    }

    for raw_layertag in disable_layertags.iter() {
        match state_registry.0.request_from_raw(raw_layertag) {
            Some(layertag) => {
                disable.0.add_layertag(layertag);
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
                added.layer_tag_container.add_layertag(layertag);
                added.revert = revertable_layertag.revertable.into();
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
                removed.layer_tag_container.add_layertag(layertag);
                removed.revert = revertable_layertag.revertable.into();
            }
            None => {
                warn!("layertag not found registry: {:?}", revertable_layertag)
            }
        }
    }

    (required, disable, added, removed)
}

/// 从数据表原始标签构建 buff 中断阶段两个标签容器
/// （所需/禁用；未注册的标签记录 warning 并跳过）。
pub fn build_buff_abort_tags(
    required_layertags: &[String],
    disable_layertags: &[String],
    state_registry: &Res<StateLayerTagRegistry>,
) -> (
    BuffAbortRequiredLayerTagContainer,
    BuffAbortDisableLayerTagContainer,
) {
    let mut required = BuffAbortRequiredLayerTagContainer::default();
    let mut disable = BuffAbortDisableLayerTagContainer::default();

    for raw_layertag in required_layertags.iter() {
        match state_registry.0.request_from_raw(raw_layertag) {
            Some(layertag) => {
                required.0.add_layertag(layertag);
            }
            None => {
                warn!("layertag not found registry: {}", raw_layertag)
            }
        }
    }

    for raw_layertag in disable_layertags.iter() {
        match state_registry.0.request_from_raw(raw_layertag) {
            Some(layertag) => {
                disable.0.add_layertag(layertag);
            }
            None => {
                warn!("layertag not found registry: {}", raw_layertag)
            }
        }
    }

    (required, disable)
}
