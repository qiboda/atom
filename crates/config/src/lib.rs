pub mod toml_diff;

/// 1. save base settings to base_config_dir if it doesn't exist
/// 2. load user_settings from user_config_dir to override base settings
/// 3. user modified settings will be saved to user_config_dir and only non base settings values will be saved
///
///
/// base and user settings are assets.
/// but final settings is a resource.
use std::{marker::PhantomData, ops::Deref, path::PathBuf};

use bevy::prelude::*;
use bevy_common_assets::toml::TomlAssetPlugin;
use project::{project_asset_root_path, project_saved_root_path};
use serde::{Deserialize, Serialize};

const BASE_EXTENSION: &'static str = ".toml";

/// settings limits:
///   1. all fields must be Optional
pub trait Settings:
    Resource + Copy + Serialize + TypePath + Default + for<'a> Deserialize<'a> + Asset
{
}

impl<T> Settings for T where
    T: Resource + Copy + Serialize + TypePath + Default + for<'a> Deserialize<'a> + Asset
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

    pub fn new() -> SettingsPath<S> {
        SettingsPath {
            base_config_dir: project_asset_root_path().join("config"),
            user_config_dir: project_saved_root_path().join("config"),
            settings: PhantomData::<S>,
        }
    }
}

#[derive(Resource, Default)]
pub struct SettingsHandle<S>
where
    S: Settings,
{
    pub base_handle: Handle<S>,
    pub user_handle: Handle<S>,
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
        let filepath = path.join(&SettingsPath::<S>::category_name());
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

        std::fs::write(
            path.join(&SettingsPath::<S>::category_name()),
            &settings_str,
        )
        .expect(&format!(
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
            .add_systems(PreUpdate, update_final_settings::<S>)
            .add_systems(Last, SettingsPlugin::<S>::persist_all)
            .add_systems(Last, SettingsPlugin::<S>::persist);
    }
}

fn startup_load_settings<S>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    paths: Res<SettingsPath<S>>,
) where
    S: Settings,
{
    let mut handle = SettingsHandle::<S>::default();
    handle.base_handle = asset_server.load(paths.get_base_config_path());
    handle.user_handle = asset_server.load(paths.get_user_config_path());
    handle.base_handle = asset_server.load(paths.get_base_config_path());
    commands.insert_resource(handle);
}

fn update_final_settings<S>(
    mut asset_event_reader: EventReader<AssetEvent<S>>,
    mut handle: ResMut<SettingsHandle<S>>,
    mut assets: ResMut<Assets<S>>,
    mut s: Res<S>,
) where
    S: Settings,
{
    asset_event_reader.read().for_each(|event| match event {
        AssetEvent::Added { id } => {
            // todo: add to user settings and final settings
            // assets.get_mut(id).unwrap().a = Some(1);
        }
        AssetEvent::Modified { id } => {
            // todo: add to user settings and final settings
        }
        AssetEvent::Removed { id } => {}
        _ => {}
    });
}

#[cfg(test)]
mod tests {
    use bevy::{asset::Asset, reflect::TypePath};
    use serde::{Deserialize, Serialize};
    use serde_merge::tmerge;

    #[derive(Serialize, Deserialize, PartialEq, Debug, Asset, TypePath)]
    struct TestSettings {
        a: Option<u32>,
        b: Option<String>,
    }

    #[test]
    fn merge() {
        let a = TestSettings {
            a: Some(1),
            b: Some("a".to_string()),
        };

        let b = TestSettings {
            a: Some(1),
            b: Some("b".to_string()),
        };

        let merge = tmerge(&a, &b).unwrap();
        assert_eq!(b, merge);
    }
}
