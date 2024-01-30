use bevy::prelude::*;

use std::marker::PhantomData;

use std::path::PathBuf;

use super::Settings;

const BASE_EXTENSION: &str = ".toml";

/// A resource that contains the paths for the settings
///     user_config_dir: The directory where the settings are saved to
///     base_config_dir: The directory where the settings are loaded defaultly from
///     And, load order is user_config_dir -> base_config_dir,
///
///     filename is TypePath::short_type_path() + ".toml"
#[derive(Reflect, Resource, Clone, Default, Debug)]
pub struct SettingsPath<S>
where
    S: Settings,
{
    pub base_config_dir: PathBuf,
    pub user_config_dir: PathBuf,
    _settings: PhantomData<S>,
}

impl<S> SettingsPath<S>
where
    S: Settings,
{
    pub fn category_name() -> &'static str {
        S::short_type_path()
    }

    pub fn extension() -> String {
        ".".to_string() + SettingsPath::<S>::category_name() + BASE_EXTENSION
    }

    pub fn get_user_config_path(&self) -> PathBuf {
        let filename = "user".to_string() + &SettingsPath::<S>::extension();
        self.user_config_dir.join(filename)
    }

    pub fn get_base_config_path(&self) -> PathBuf {
        let filename = "base".to_string() + &SettingsPath::<S>::extension();
        self.base_config_dir.join(filename)
    }
}
