use bevy::prelude::*;
use cfg::{global::TbGlobal, item::TbItem, unit::TbUnit, Tables};
use datatables::{tables_ext::TableReader, DataTablePlugin, TableLoadingStates};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DataTablePlugin)
        .add_systems(
            Update,
            (multiple_reader, print_table_data).run_if(in_state(TableLoadingStates::Loaded)),
        )
        .run();
}

fn print_table_data(
    item_table: Res<Assets<TbItem>>,
    tables: Res<Tables>,
    mut event_writer: EventWriter<AppExit>,
) {
    if let Some(item_table) = item_table.get(tables.tb_item.id()) {
        println!("{:?}", item_table.get(&10000));
        event_writer.send(AppExit::Success);
    } else {
        println!("loading");
    }
}

fn multiple_reader(table_reader: TableReader<TbUnit>, s: TableReader<TbGlobal>) {
    if let Some(tb) = table_reader.get_row(&10001) {
        println!("{:?}", tb);
    }
    if let Some(tb) = s.get_data() {
        println!("{:?}", tb);
    }
}
