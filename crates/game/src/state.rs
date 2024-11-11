use bevy::prelude::*;
use datatables::TableLoadingState;
use settings::SettingsLoadStatus;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, States)]
pub enum GameState {
    #[default]
    LoadData,
    InitGame,
    RunGame,
}

#[derive(Debug, Default)]
pub struct GameStatePlugin;

impl Plugin for GameStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>().add_systems(
            First,
            (
                // 跨一帧执行。
                start_run_game.run_if(in_state(GameState::InitGame)),
                wait_load_startup_assets_over.run_if(in_state(GameState::LoadData)),
            )
                .chain(),
        );
    }
}

fn wait_load_startup_assets_over(
    mut state: ResMut<NextState<GameState>>,
    table_state: Res<State<TableLoadingState>>,
    settings_status: Res<SettingsLoadStatus>,
) {
    info!("wait_load_startup_assets_over");
    if table_state.get() == &TableLoadingState::Loaded && settings_status.all_loaded() {
        state.set(GameState::InitGame);
        info!("game state to InitGame");
    }
}

fn start_run_game(mut state: ResMut<NextState<GameState>>) {
    state.set(GameState::RunGame);
    info!("start_run_game");
}
