use atom_camera::setting::CameraSetting;
use bevy::prelude::*;
use datatables::TableLoadingState;
use settings::load::{SettingLoadStage, SettingLoadStageWrap};

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, States)]
pub enum GameState {
    #[default]
    LoadData,
    InitGame,
    RunGame,
}

pub fn next_to_init_game_state(
    mut next_game_state: ResMut<NextState<GameState>>,
    game_state: Res<State<GameState>>,
    table_loading_state: Res<State<TableLoadingState>>,
    // load_stage: Res<SettingLoadStageWrap<CameraSetting>>,
) {
    if *game_state == GameState::LoadData && *table_loading_state == TableLoadingState::Loaded
    // && load_stage.setting_load_stage == SettingLoadStage::LoadOver
    {
        next_game_state.set(GameState::InitGame);
    }
}
