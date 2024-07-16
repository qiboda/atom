use atom_camera::CameraManagerPlugin;
use atom_internal::plugins::AtomDefaultPlugins;
use atom_utils::follow::TransformFollowPlugin;
use avian3d::{debug_render::PhysicsDebugPlugin, PhysicsPlugins};
use bevy::{
    app::Plugin,
    core::{TaskPoolOptions, TaskPoolPlugin, TaskPoolThreadAssignmentPolicy},
    log::LogPlugin,
    prelude::*,
    DefaultPlugins,
};
use bevy_console::{AddConsoleCommand, ConsolePlugin};
use bevy_tnua::controller::TnuaControllerPlugin;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use datatables::DataTablePlugin;
use input::setting::{
    input_setting_persist_command, update_player_input, PlayerAction, PlayerInputSetting,
    PlayerInputSettingPersistCommand,
};
use leafwing_input_manager::plugin::{InputManagerPlugin, InputManagerSubsystemPlugin};
use log_layers::{file_layer, LogLayersPlugin};
use scene::init_scene;
use settings::{setting_path::SettingsPath, SettingPlugin, SettingSourceConfig, SettingsPlugin};
use state::{next_to_init_game_state, GameState};

pub mod ai;
pub mod damage;
pub mod input;
pub mod items;
pub mod projectile;
pub mod scene;
pub mod state;
pub mod unit;

#[derive(Debug, Default)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {

        app.add_plugins(AtomDefaultPlugins)
            .add_plugins(SettingPlugin::<PlayerInputSetting> {
                paths: SettingsPath::default(),
            })
            .add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_plugins(TransformFollowPlugin)
            .add_plugins(CameraManagerPlugin)
            .add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()))
            .add_plugins((
                TnuaControllerPlugin::default(),
                TnuaAvian3dPlugin::default(),
            ))
            .insert_state(GameState::default())
            .add_console_command::<PlayerInputSettingPersistCommand, _>(
                input_setting_persist_command,
            )
            .add_systems(OnEnter(GameState::InitGame), init_scene)
            .add_systems(
                Update,
                update_player_input.run_if(in_state(GameState::RunGame)),
            )
            .add_systems(Last, next_to_init_game_state);
    }
}

// todo: add tune.
