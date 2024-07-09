use bevy::{log::LogPlugin, prelude::*};
use log_layers::{file_layer, LogLayersPlugin};
use settings::{SettingSourceConfig, SettingsPlugin};
use terrain::TerrainSubsystemPlugin;

pub fn main() {
    let mut app = App::new();

    app.add_plugins(SettingsPlugin {
        game_source_config: SettingSourceConfig {
            source_id: "config_terrain".into(),
            base_path: "config/terrain".into(),
        },
        user_source_config: SettingSourceConfig {
            source_id: "config_terrain".into(),
            base_path: "config/terrain".into(),
        },
    })
    .add_plugins((
        LogLayersPlugin::default().add_layer(file_layer::file_layer),
        DefaultPlugins.set(LogPlugin {
            custom_layer: LogLayersPlugin::get_layer,
            ..default()
        }),
    ))
    .add_plugins(TerrainSubsystemPlugin)
    .run();
}
