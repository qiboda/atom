use std::path::PathBuf;

use atom_camera::CameraManagerPlugin;
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
use datatables::DataTablePlugin;
use input::setting::{
    input_setting_persist_command, PlayerAction, PlayerInputSetting,
    PlayerInputSettingPersistCommand,
};
use leafwing_input_manager::plugin::InputManagerPlugin;
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
        let task_pool_plugin = TaskPoolPlugin {
            task_pool_options: TaskPoolOptions {
                async_compute: TaskPoolThreadAssignmentPolicy {
                    min_threads: 1,
                    max_threads: usize::MAX,
                    percent: 0.25,
                },
                ..default()
            },
        };

        let settings_plugin = SettingsPlugin {
            game_source_config: SettingSourceConfig {
                source_id: "game_settings".into(),
                base_path: "config".into(),
            },
            user_source_config: SettingSourceConfig {
                source_id: "user_settings".into(),
                base_path: "config".into(),
            },
        };

        app.add_plugins(settings_plugin)
            .add_plugins((
                LogLayersPlugin::default().add_layer(file_layer::file_layer),
                DefaultPlugins
                    .set(LogPlugin {
                        custom_layer: LogLayersPlugin::get_layer,
                        ..default()
                    })
                    .set(task_pool_plugin),
            ))
            .add_plugins(SettingPlugin::<PlayerInputSetting> {
                paths: SettingsPath::default(),
            })
            // .add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_plugins(ConsolePlugin)
            .add_plugins(DataTablePlugin)
            .add_plugins(TransformFollowPlugin)
            .add_plugins(CameraManagerPlugin)
            .add_plugins((PhysicsPlugins::default(), PhysicsDebugPlugin::default()))
            .insert_state(GameState::default())
            .add_console_command::<PlayerInputSettingPersistCommand, _>(
                input_setting_persist_command,
            )
            .add_systems(OnEnter(GameState::InitGame), init_scene)
            .add_systems(Last, next_to_init_game_state);
    }
}
