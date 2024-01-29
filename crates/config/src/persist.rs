use std::{marker::PhantomData, ops::Deref, path::PathBuf};

use crate::{setting_path::SettingsPath, Settings};
use bevy::prelude::*;

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

pub(crate) fn persist_all<S>(
    settings: Res<S>,
    paths: Res<SettingsPath<S>>,
    mut reader: EventReader<PersistAllSettings>,
) where
    S: Settings,
{
    if !reader.is_empty() {
        reader.read().for_each(|event| {
            let mut path = paths.user_config_dir.clone();
            if let Some(overide_path) = event.override_user_path.clone() {
                path = overide_path;
            }

            persist_internal(&settings, path);
        });
    }
}

pub(crate) fn persist<S>(
    settings: Res<S>,
    paths: Res<SettingsPath<S>>,
    mut reader: EventReader<PersistSettings<S>>,
) where
    S: Settings,
{
    if !reader.is_empty() {
        reader.read().for_each(|event| {
            let mut path = paths.user_config_dir.clone();
            if let Some(overide_path) = event.override_user_path.clone() {
                path = overide_path;
            }
            persist_internal(&settings, path);
        });
    }
}

fn persist_internal<S>(settings: &Res<S>, path: PathBuf)
where
    S: Settings,
{
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
