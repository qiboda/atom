use bevy::{
    app::{App, Plugin},
    state::state::OnEnter,
};
use datatables::TableLoadingState;

#[derive(Debug, Default)]
pub struct InventoryPlugin;

impl Plugin for InventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(TableLoadingState::Loaded), init_inventory);
    }
}

fn init_inventory() {}
