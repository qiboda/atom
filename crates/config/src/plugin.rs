use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;

use crate::load::{
    is_load_setting, refresh_final_settings, startup_load_settings, SettingLoadStageWrap,
    SettingsHandle,
};
use crate::persist::{persist, persist_all, PersistAllSettings, PersistSettings};
use crate::setting_path::SettingsPath;
use crate::Settings;

/// Global settings config for the settings plugin
#[derive(Debug, Default)]
pub struct SettingsPlugin<S>
where
    S: Settings,
{
    pub(crate) paths: SettingsPath<S>,
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
            .init_resource::<SettingLoadStageWrap<S>>()
            .init_resource::<S>()
            .add_event::<PersistSettings<S>>()
            .add_event::<PersistAllSettings>()
            .add_systems(
                Startup,
                startup_load_settings::<S>.run_if(is_load_setting::<S>),
            )
            .add_systems(PreUpdate, refresh_final_settings::<S>)
            .add_systems(Last, persist_all::<S>)
            .add_systems(Last, persist::<S>);
    }
}
