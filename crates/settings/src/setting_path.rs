use bevy::prelude::*;

use std::marker::PhantomData;

use std::path::PathBuf;

use super::Setting;

const BASE_EXTENSION: &str = ".toml";

/// 单独某一个配置的路径，覆盖全局的默认值。
#[derive(Reflect, Resource, Clone, Default, Debug)]
pub struct SettingsPath<S>
where
    S: Setting,
{
    pub game_config_dir: Option<PathBuf>,
    pub user_config_dir: Option<PathBuf>,
    _settings: PhantomData<S>,
}

impl<S> SettingsPath<S>
where
    S: Setting,
{
    fn category_name() -> &'static str {
        S::short_type_path()
    }

    pub fn extension() -> String {
        ".".to_string() + SettingsPath::<S>::category_name() + BASE_EXTENSION
    }

    pub fn get_user_config_path(&self) -> Option<PathBuf> {
        let filename = "user".to_string() + &SettingsPath::<S>::extension();
        self.user_config_dir.as_ref().map(|x| x.join(filename))
    }

    pub fn get_game_config_path(&self) -> Option<PathBuf> {
        let filename = "game".to_string() + &SettingsPath::<S>::extension();
        self.game_config_dir.as_ref().map(|x| x.join(filename))
    }
}
