use bevy::prelude::*;
use datatables::TableLoadingState;
use settings::SettingsLoadStatus;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    LoadingStartupAssets,
    AppRunning,
}

#[derive(Debug, Default)]
pub struct AppStatePlugin;

impl Plugin for AppStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>().add_systems(
            Update,
            wait_load_startup_assets_over.run_if(in_state(AppState::LoadingStartupAssets)),
        );
    }
}

fn wait_load_startup_assets_over(
    mut state: ResMut<NextState<AppState>>,
    table_state: Res<State<TableLoadingState>>,
    settings_status: Res<SettingsLoadStatus>,
) {
    if table_state.get() == &TableLoadingState::Loaded && settings_status.all_loaded() {
        state.set(AppState::AppRunning);
        info!("app state to AppRunning");
    }
}
