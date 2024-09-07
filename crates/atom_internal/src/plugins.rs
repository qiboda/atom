use atom_shader_lib::AtomShaderLibPluginGroups;
use bevy::{
    app::{PluginGroup, PluginGroupBuilder},
    asset::AssetPlugin,
    color::Color,
    core::{TaskPoolOptions, TaskPoolPlugin, TaskPoolThreadAssignmentPolicy},
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    log::LogPlugin,
    text::TextStyle,
    utils::default,
    DefaultPlugins,
};
use bevy_console::ConsolePlugin;
use bevy_debug_grid::DebugGridPlugin;
use datatables::DataTablePlugin;
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
                    source_id: "config".into(),
                    base_path: root_path.join("config"),
                },
                user_source_config: SettingSourceConfig {
                    source_id: "config".into(),
                    base_path: root_path.join("config"),
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
            .add_group(AtomShaderLibPluginGroups)
            .add(ConsolePlugin)
            .add(StateMachinePlugin)
            .add(DataTablePlugin)
            // .add(DebugGridPlugin::without_floor_grid())
            .add(AppStatePlugin)
            .add(FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextStyle {
                        // Here we define size of our overlay
                        font_size: 50.0,
                        // We can also change color of the overlay
                        color: Color::srgb(0.0, 1.0, 0.0),
                        // If we want, we can use a custom font
                        font: default(),
                    },
                },
            });

        group
    }
}
