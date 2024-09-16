use atom_shader_lib::AtomShaderLibPluginGroups;
use avian3d::{sync::SyncPlugin, PhysicsPlugins};
use bevy::{
    app::{FixedUpdate, PluginGroup, PluginGroupBuilder, PostUpdate},
    asset::AssetPlugin,
    core::{TaskPoolOptions, TaskPoolPlugin, TaskPoolThreadAssignmentPolicy},
    log::LogPlugin,
    prelude::*,
    DefaultPlugins,
};
use bevy_console::ConsolePlugin;
use datatables::DataTablePlugin;
use log_layers::LogLayersPlugin;
use seldom_state::StateMachinePlugin;
use settings::{SettingSourceConfig, SettingsPlugin};

#[derive(Debug, Default)]
pub struct AtomClientPlugins;

impl PluginGroup for AtomClientPlugins {
    fn build(self) -> PluginGroupBuilder {
        project::log_all_path();

        let root_path = project::project_root_path();
        let asset_root_path = project::project_asset_root_path();
        let processed_asset_root_path = project::project_processed_asset_root_path();

        let mut group = PluginGroupBuilder::start::<Self>();
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
            .add(ConsolePlugin)
            .add(StateMachinePlugin)
            .add(DataTablePlugin)
            .add_group(AtomShaderLibPluginGroups)
            .add_group(
                PhysicsPlugins::new(FixedUpdate)
                    .build()
                    .disable::<SyncPlugin>(),
            )
            .add(SyncPlugin::new(PostUpdate));
        // .add(FpsOverlayPlugin {
        //     config: FpsOverlayConfig {
        //         text_config: TextStyle {
        //             // Here we define size of our overlay
        //             font_size: 50.0,
        //             // We can also change color of the overlay
        //             color: Color::srgb(0.0, 1.0, 0.0),
        //             // If we want, we can use a custom font
        //             font: default(),
        //         },
        //     },
        // });

        group
    }
}

#[derive(Debug, Default)]
pub struct AtomServerPlugins;

impl PluginGroup for AtomServerPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
    }
}

#[derive(Debug, Default)]
pub struct AtomSharedPlugins;

impl PluginGroup for AtomSharedPlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
    }
}

#[derive(Debug, Default)]
pub struct AtomHostServerPlugins;

impl PluginGroup for AtomHostServerPlugins {
    fn build(self) -> PluginGroupBuilder {
        let mut group = PluginGroupBuilder::start::<Self>();
        group = group
            .add_group(AtomSharedPlugins)
            .add_group(AtomServerPlugins)
            .add_group(AtomClientPlugins);
        group
    }
}
