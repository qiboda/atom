//! 状态层（StateLayer）：实体状态层标签容器与全局标签注册表。

use atom_datatables::{TablesLoadedEvent, layertag::TbLayerTag, tables_system_param::TableReader};
use atom_layertag::{count_container::CountLayerTagContainer, registry::LayerTagRegistry};
use bevy::{
    ecs::message::MessageReader,
    prelude::{Component, ResMut, Resource},
    reflect::Reflect,
};

/// 实体状态层标签容器组件：持有该实体当前的全部状态层标签。
#[derive(Component, Default, Debug, Reflect)]
pub struct StateLayerTagContainer(pub CountLayerTagContainer);

/// 全局状态层标签注册表：从数据表加载原始标签字符串。
#[derive(Resource, Default, Debug, Reflect)]
pub struct StateLayerTagRegistry(pub LayerTagRegistry);

// TODO: 设置整个游戏的State，保证执行顺序。
/// 数据表加载完成后初始化状态层标签注册表。
pub fn init_state_layertag_registry(
    mut event_reader: MessageReader<TablesLoadedEvent>,
    table: TableReader<TbLayerTag>,
    mut registry: ResMut<StateLayerTagRegistry>,
) {
    if event_reader.read().len() > 0
        && let Some(list) = table.get_data_list_in_map_table()
    {
        registry.0.clear();
        list.iter().for_each(|value| {
            registry.0.register_raw(&value.raw_layertag);
        });
    }
}
