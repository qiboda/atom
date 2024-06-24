use bevy::prelude::*;
use cfg::{item::TbItem, Tables};
use datatables::DataTablePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DataTablePlugin)
        .add_systems(Update, print_table_data)
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
