//! 技能状态层标签组件：技能各阶段所需/禁用/增删的标签集合。

use atom_layertag::count_container::CountLayerTagContainer;
use bevy::prelude::*;

/// 技能开始所需的状态层标签容器（全部满足才可开始）。
#[derive(Component, Debug, Default, Reflect)]
pub struct AbilityStartRequiredLayerTagContainer(pub CountLayerTagContainer);

/// 技能开始禁用的状态层标签容器（存在任一即不可开始）。
#[derive(Component, Debug, Default, Reflect)]
pub struct AbilityStartDisableLayerTagContainer(pub CountLayerTagContainer);

/// 技能中断所需的状态层标签容器（全部满足才可中断）。
#[derive(Component, Debug, Default, Reflect)]
pub struct AbilityAbortRequiredLayerTagContainer(pub CountLayerTagContainer);

/// 技能中断禁用的状态层标签容器（存在任一即不可中断）。
#[derive(Component, Debug, Default, Reflect)]
pub struct AbilityAbortDisableLayerTagContainer(pub CountLayerTagContainer);

/// 技能结束后是否回滚状态层标签。
#[derive(Debug, Default, Reflect, PartialEq, Eq)]
pub enum AbilityLayerTagContainerRevert {
    /// 不回滚（默认）。
    #[default]
    No,
    /// 回滚。
    Yes,
}

impl From<bool> for AbilityLayerTagContainerRevert {
    fn from(value: bool) -> Self {
        if value {
            AbilityLayerTagContainerRevert::Yes
        } else {
            AbilityLayerTagContainerRevert::No
        }
    }
}

/// 技能开始时要添加的状态层标签集合（带回滚标记）。
#[derive(Component, Debug, Default, Reflect)]
pub struct AbilityAddedLayerTagContainer {
    /// 要添加的标签集合。
    pub layer_tag_container: CountLayerTagContainer,
    /// 结束后是否回滚。
    pub revert: AbilityLayerTagContainerRevert,
}

/// 技能开始时要移除的状态层标签集合（带回滚标记）。
#[derive(Component, Debug, Default, Reflect)]
pub struct AbilityRemovedLayerTagContainer {
    /// 要移除的标签集合。
    pub layer_tag_container: CountLayerTagContainer,
    /// 结束后是否回滚。
    pub revert: AbilityLayerTagContainerRevert,
}
