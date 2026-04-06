use atom_cfg::{Tables, global::TbGlobal, item::TbItem, unit::TbNpc};
use atom_datatables::{DataTablePlugin, TableLoadingState, tables_system_param::TableReader};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DataTablePlugin)
        .add_systems(
            Update,
            (multiple_reader, print_table_data).run_if(in_state(TableLoadingState::Loaded)),
        )
        .run();
}

fn print_table_data(
    item_table: Res<Assets<TbItem>>,
    tables: Res<Tables>,
    mut event_writer: MessageWriter<AppExit>,
) {
    if let Some(item_table) = item_table.get(tables.tb_item.id()) {
        println!("{:?}", item_table.get(&10000));
        event_writer.write(AppExit::Success);
    } else {
        println!("loading");
    }
}

fn multiple_reader(table_reader: TableReader<TbNpc>, s: TableReader<TbGlobal>) {
    if let Some(tb) = table_reader.get_row(&10001) {
        println!("{:?}", tb);
    }
    if let Some(tb) = s.get_data() {
        println!("{:?}", tb);
    }
}
