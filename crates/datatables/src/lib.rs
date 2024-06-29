pub mod tables_system_param;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};

use atom_utils::asset_barrier::{AllAssetBarrier, AssetBarrierStatus};
use cfg::{TableAssetsPlugin, Tables, TablesLoadedEvent};

#[derive(Debug, Default, Hash, PartialEq, Eq, Clone, States)]
pub enum TableLoadingStates {
    #[default]
    Wait,
    Loading,
    Loaded,
}

#[derive(Debug, Default, Resource, Deref)]
pub struct TablesBarrierStatus(pub(crate) AssetBarrierStatus);

#[derive(Default)]
pub struct DataTablePlugin;

impl Plugin for DataTablePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TableAssetsPlugin)
            .insert_resource(AllAssetBarrier::default())
            .init_resource::<Tables>()
            .insert_state(TableLoadingStates::default())
            .add_systems(Startup, start_load_tables)
            .add_systems(
                PreUpdate,
                update_table_loading_state.run_if(in_state(TableLoadingStates::Loading)),
            )
            .add_systems(
                OnExit(TableLoadingStates::Loading),
                clear_table_loading_status,
            );
    }
}

fn start_load_tables(
    mut commands: Commands,
    mut all_asset_barrier: ResMut<AllAssetBarrier>,
    mut table_loading_states: ResMut<NextState<TableLoadingStates>>,
    asset_server: Res<AssetServer>,
) {
    if let Some((barrier, guard)) = all_asset_barrier.create_asset_barrier("Table".to_owned()) {
        info!("start_load_tables");
        table_loading_states.set(TableLoadingStates::Loading);

        commands.insert_resource(Tables::new(asset_server, "datatables/".into(), guard));
        let future = barrier.wait_async();

        let loading_state = Arc::new(AtomicBool::new(false));
        commands.insert_resource(TablesBarrierStatus(AssetBarrierStatus {
            barrier_key: "Tables".to_owned(),
            barrier_end: loading_state.clone(),
        }));

        // await the `AssetBarrierFuture`.
        AsyncComputeTaskPool::get()
            .spawn(async move {
                future.await;
                // Notify via `AsyncLoadingState`
                loading_state.store(true, Ordering::Release);
            })
            .detach();
    }
}

fn update_table_loading_state(
    table_asset_barrier_state: Res<TablesBarrierStatus>,
    mut table_loading_states: ResMut<NextState<TableLoadingStates>>,
    mut event_writer: EventWriter<TablesLoadedEvent>,
    tables: Res<Tables>,
) {
    if table_asset_barrier_state
        .0
        .barrier_end
        .load(Ordering::Acquire)
    {
        info!("update_table_loading_state");
        table_loading_states.set(TableLoadingStates::Loaded);

        let e = TablesLoadedEvent {
            asset_handles: tables.table_handle_map.values().cloned().collect(),
        };
        event_writer.send(e);
    }
}

fn clear_table_loading_status(
    mut commands: Commands,
    mut all_asset_barrier: ResMut<AllAssetBarrier>,
    tables_barrier_state: Res<TablesBarrierStatus>,
) {
    info!("clear_table_loading_status");
    all_asset_barrier.remove_asset_barrier(&tables_barrier_state.barrier_key);
    commands.remove_resource::<TablesBarrierStatus>();
}
