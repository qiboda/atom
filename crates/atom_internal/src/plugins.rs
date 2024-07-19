use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    asset::AssetPlugin,
    core::{TaskPoolOptions, TaskPoolPlugin, TaskPoolThreadAssignmentPolicy},
    log::LogPlugin,
    utils::default,
    DefaultPlugins,
};
use bevy_console::ConsolePlugin;
use datatables::DataTablePlugin;
use leafwing_input_manager::plugin::InputManagerSubsystemPlugin;
use log_layers::{file_layer, LogLayersPlugin};
use seldom_state::StateMachinePlugin;
use settings::{SettingSourceConfig, SettingsPlugin};

use crate::app_state::AppStatePlugin;

#[derive(Debug, Default)]
pub struct AtomDefaultPlugins;

impl PluginGroup for AtomDefaultPlugins {
    fn build(self) -> PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();

        project::log_all_path();

        let root_path = project::project_root_path();
        let asset_root_path = project::project_asset_root_path();
        let processed_asset_root_path = project::project_processed_asset_root_path();
        group = group
            .add(SettingsPlugin {
                game_source_config: SettingSourceConfig {
                    source_id: "config_terrain".into(),
                    base_path: root_path.join("config/terrain"),
                },
                user_source_config: SettingSourceConfig {
                    source_id: "config_terrain".into(),
                    base_path: root_path.join("config/terrain"),
                },
            })
            .add(LogLayersPlugin::default().add_layer(file_layer::file_layer))
            .add_group(
                DefaultPlugins
                    .set(LogPlugin {
                        custom_layer: LogLayersPlugin::get_layer,
                        ..default()
                    })
                    .set(TaskPoolPlugin {
                        task_pool_options: TaskPoolOptions {
                            async_compute: TaskPoolThreadAssignmentPolicy {
                                min_threads: 1,
                                max_threads: usize::MAX,
                                percent: 0.25,
                            },
                            ..default()
                        },
                    })
                    .set(AssetPlugin {
                        file_path: asset_root_path.to_str().unwrap().to_string(),
                        processed_file_path: processed_asset_root_path
                            .to_str()
                            .unwrap()
                            .to_string(),
                        ..default()
                    }),
            )
            .add(InputManagerSubsystemPlugin)
            .add(ConsolePlugin)
            .add(StateMachinePlugin)
            .add(DataTablePlugin)
            // .add(DebugGridPlugin::without_floor_grid())
            .add(AppStatePlugin);

        group
    }
}
