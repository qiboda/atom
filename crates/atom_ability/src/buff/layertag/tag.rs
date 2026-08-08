//! Buff 状态层标签组件：buff 各阶段所需/禁用/增删的标签集合。

use atom_layertag::count_container::CountLayerTagContainer;
use bevy::prelude::*;

/// Buff 开始所需的状态层标签容器（全部满足才可开始）。
#[derive(Component, Debug, Default, Reflect)]
pub struct BuffStartRequiredLayerTagContainer(pub CountLayerTagContainer);

/// Buff 开始禁用的状态层标签容器（存在任一即不可开始）。
#[derive(Component, Debug, Default, Reflect)]
pub struct BuffStartDisableLayerTagContainer(pub CountLayerTagContainer);

/// Buff 中断所需的状态层标签容器（全部满足才可中断）。
#[derive(Component, Debug, Default, Reflect)]
pub struct BuffAbortRequiredLayerTagContainer(pub CountLayerTagContainer);

/// Buff 中断禁用的状态层标签容器（存在任一即不可中断）。
#[derive(Component, Debug, Default, Reflect)]
pub struct BuffAbortDisableLayerTagContainer(pub CountLayerTagContainer);

/// Buff 结束后是否回滚状态层标签。
#[derive(Debug, Default, Reflect, PartialEq, Eq)]
pub enum BuffLayerTagContainerRevert {
    /// 不回滚（默认）。
    #[default]
    No,
    /// 回滚。
    Yes,
}

impl From<bool> for BuffLayerTagContainerRevert {
    fn from(value: bool) -> Self {
        if value {
            BuffLayerTagContainerRevert::Yes
        } else {
            BuffLayerTagContainerRevert::No
        }
    }
}

/// Buff 开始时要添加的状态层标签集合（带回滚标记）。
#[derive(Component, Debug, Default, Reflect)]
pub struct BuffAddedLayerTagContainer {
    /// 要添加的标签集合。
    pub layer_tag_container: CountLayerTagContainer,
    /// 结束后是否回滚。
    pub revert: BuffLayerTagContainerRevert,
}

/// Buff 开始时要移除的状态层标签集合（带回滚标记）。
#[derive(Component, Debug, Default, Reflect)]
pub struct BuffRemovedLayerTagContainer {
    /// 要移除的标签集合。
    pub layer_tag_container: CountLayerTagContainer,
    /// 结束后是否回滚。
    pub revert: BuffLayerTagContainerRevert,
}
