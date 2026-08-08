//! 状态层（StateLayer）：实体状态层标签容器与全局标签注册表。

use atom_data::DataRegistry;
use atom_layertag::{count_container::CountLayerTagContainer, registry::LayerTagRegistry};
use bevy::{
    prelude::{Component, Res, ResMut, Resource},
    reflect::Reflect,
};

use crate::config::LayerTagConfig;

/// 实体状态层标签容器组件：持有该实体当前的全部状态层标签。
#[derive(Component, Default, Debug, Reflect)]
pub struct StateLayerTagContainer(pub CountLayerTagContainer);

/// 全局状态层标签注册表：从数据表加载原始标签字符串。
#[derive(Resource, Default, Debug, Reflect)]
pub struct StateLayerTagRegistry(pub LayerTagRegistry);

/// 从 `LayerTagConfig` 数据表初始化状态层标签注册表：表已加载时全量重建
/// （clear + 逐行注册 `raw_layertag`）；未加载时不改动注册表。
pub fn init_state_layertag_registry(
    registry: Res<DataRegistry>,
    mut state_registry: ResMut<StateLayerTagRegistry>,
) {
    let Some(table) = registry.table::<LayerTagConfig>() else {
        return;
    };

    state_registry.0.clear();
    table.iter().for_each(|row| {
        state_registry.0.register_raw(&row.raw_layertag);
    });
}
