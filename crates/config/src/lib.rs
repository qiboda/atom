use std::{marker::PhantomData, ops::Deref, path::PathBuf};

use bevy::prelude::*;
use project::{project_asset_root_path, project_saved_root_path};
use serde::{Deserialize, Serialize};

pub trait Settings:
    Resource + Copy + Serialize + TypePath + Default + for<'a> Deserialize<'a>
{
}

impl<T> Settings for T where
    T: Resource + Copy + Serialize + TypePath + Default + for<'a> Deserialize<'a>
{
}

#[derive(Event, Default)]
pub struct PersistSettings<S>
where
    S: Settings,
{
    pub override_user_path: Option<PathBuf>,
    settings: PhantomData<S>,
}

#[derive(Event, Default)]
pub struct PersistAllSettings {
    pub override_user_path: Option<PathBuf>,
}

/// A resource that contains the paths for the settings
///     user_config_dir: The directory where the settings are saved to
///     base_config_dir: The directory where the settings are loaded defaultly from
///     And, load order is user_config_dir -> base_config_dir,
///
///     filename is TypePath::short_type_path() + ".toml"
#[derive(Reflect, Resource, Clone)]
pub struct SettingsPath<S>
where
    S: Settings,
{
    pub base_config_dir: PathBuf,
    pub user_config_dir: PathBuf,
    settings: PhantomData<S>,
}

impl<S> SettingsPath<S>
where
    S: Settings,
{
    pub fn filename() -> &'static str {
        S::short_type_path()
    }

    pub fn new() -> SettingsPath<S> {
        SettingsPath {
            base_config_dir: project_asset_root_path().join("config"),
            user_config_dir: project_saved_root_path().join("config"),
            settings: PhantomData::<S>,
        }
    }
}

/// Global settings config for the settings plugin
pub struct SettingsPlugin<S>
where
    S: Settings,
{
    paths: SettingsPath<S>,
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

    pub fn resource(&self) -> S {
        self.load().unwrap_or_default()
    }

    fn load_in_path(&self, path: &PathBuf) -> Option<S> {
        let filepath = path.join(&SettingsPath::<S>::filename());
        if !filepath.exists() {
            info!(
                "Couldn't find the settings file at {:?}",
                filepath.as_os_str()
            );
            return None;
        }

        let settings_string = std::fs::read_to_string(&filepath);
        if settings_string.is_err() {
            error!(
                "Couldn't read the settings file at {:?}",
                filepath.as_os_str()
            );
            return None;
        }

        let settings_string = toml::from_str(&settings_string.unwrap());
        if settings_string.is_err() {
            error!(
                "Couldn't deserialize the settings file at {:?}",
                filepath.as_os_str()
            );
            return None;
        }

        settings_string.unwrap()
    }

    fn load(&self) -> Option<S> {
        if let Some(s) = self.load_in_path(&self.paths.user_config_dir) {
            return Some(s);
        }

        if let Some(s) = self.load_in_path(&self.paths.base_config_dir) {
            return Some(s);
        }

        None
    }

    fn persist_internal(settings: &Res<S>, path: PathBuf) {
        if std::fs::create_dir_all(&path).is_err() {
            error!(
                "Couldn't create the settings directory at {:?}",
                path.as_os_str()
            );
        }

        let settings_str = toml::to_string(&settings.deref()).expect(&format!(
            "Couldn't serialize the settings to toml {:?}",
            path.as_os_str()
        ));

        std::fs::write(path.join(&SettingsPath::<S>::filename()), &settings_str).expect(&format!(
            "couldn't persist the settings {:?} while trying to write the string tg disk",
            path.as_os_str()
        ));
    }

    fn persist_all(
        settings: Res<S>,
        paths: Res<SettingsPath<S>>,
        mut reader: EventReader<PersistAllSettings>,
    ) {
        if !reader.is_empty() {
            reader.read().for_each(|event| {
                let mut path = paths.user_config_dir.clone();
                if let Some(overide_path) = event.override_user_path.clone() {
                    path = overide_path;
                }

                Self::persist_internal(&settings, path);
            });
        }
    }

    fn persist(
        settings: Res<S>,
        paths: Res<SettingsPath<S>>,
        mut reader: EventReader<PersistSettings<S>>,
    ) {
        if !reader.is_empty() {
            reader.read().for_each(|event| {
                let mut path = paths.user_config_dir.clone();
                if let Some(overide_path) = event.override_user_path.clone() {
                    path = overide_path;
                }
                Self::persist_internal(&settings, path);
            });
        }
    }
}

impl<S> Plugin for SettingsPlugin<S>
where
    S: Resource + Copy + Serialize + TypePath + Default + for<'a> Deserialize<'a>,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(self.resource())
            .insert_resource(self.paths.clone())
            .add_event::<PersistSettings<S>>()
            .add_event::<PersistAllSettings>()
            .add_systems(Last, SettingsPlugin::<S>::persist_all)
            .add_systems(Last, SettingsPlugin::<S>::persist);
    }
}
