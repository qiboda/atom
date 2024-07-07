use std::path::PathBuf;

use bevy::{
    app::Plugin,
    core::{TaskPoolOptions, TaskPoolPlugin, TaskPoolThreadAssignmentPolicy},
    log::LogPlugin,
    prelude::*,
    DefaultPlugins,
};
use bevy_console::{AddConsoleCommand, ConsolePlugin};
use input::setting::{
    input_setting_persist_command, InputSetting, InputSettingPersistCommand, PlayerAction,
};
use leafwing_input_manager::plugin::InputManagerPlugin;
use log_layers::{file_layer, LogLayersPlugin};
use settings::{setting_path::SettingsPath, SettingPlugin, SettingSourceConfig, SettingsPlugin};

pub mod ai;
pub mod damage;
pub mod input;
pub mod items;
pub mod projectile;
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
            .add_plugins(SettingPlugin::<InputSetting> {
                paths: SettingsPath {
                    game_config_dir: Some(PathBuf::from("")),
                    user_config_dir: Some(PathBuf::from("")),
                    ..Default::default()
                },
            })
            .add_plugins(InputManagerPlugin::<PlayerAction>::default())
            .add_plugins(ConsolePlugin)
            .add_console_command::<InputSettingPersistCommand, _>(input_setting_persist_command);
    }
}
