use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;

use crate::load::{refresh_final_settings, startup_load_settings, SettingsHandle};
use crate::persist::{persist, persist_all, PersistAllSettings, PersistSettings};
use crate::setting_path::SettingsPath;
use crate::Settings;

/// Global settings config for the settings plugin
pub struct SettingsPlugin<S>
where
    S: Settings,
{
    pub(crate) paths: SettingsPath<S>,
}

impl<S> SettingsPlugin<S>
where
    S: Settings,
{
    pub fn new() -> Self {
        Self {
            paths: SettingsPath::new(),
        }
    }
}

impl<S> Plugin for SettingsPlugin<S>
where
    S: Settings,
{
    fn build(&self, app: &mut App) {
        let extension = SettingsPath::<S>::extension();

        app.add_plugins(TomlAssetPlugin::<S>::new(&[extension.leak()]))
            .insert_resource(self.paths.clone())
            .init_resource::<SettingsHandle<S>>()
            .init_resource::<S>()
            .add_event::<PersistSettings<S>>()
            .add_event::<PersistAllSettings>()
            .add_systems(Startup, startup_load_settings::<S>)
            .add_systems(PreUpdate, refresh_final_settings::<S>)
            .add_systems(Last, persist_all::<S>)
            .add_systems(Last, persist::<S>);
    }
}
