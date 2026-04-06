use atom_datatables::{TablesLoadedEvent, layertag::TbLayerTag, tables_system_param::TableReader};
use atom_layertag::{count_container::CountLayerTagContainer, registry::LayerTagRegistry};
use bevy::{
    ecs::message::MessageReader,
    prelude::{Component, ResMut, Resource},
    reflect::Reflect,
};

#[derive(Component, Default, Debug, Reflect)]
pub struct StateLayerTagContainer(pub CountLayerTagContainer);

#[derive(Resource, Default, Debug, Reflect)]
pub struct StateLayerTagRegistry(pub LayerTagRegistry);

// TODO: 设置整个游戏的State，保证执行顺序。
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
