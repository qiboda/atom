use bevy::{asset::LoadState, prelude::*};
use serde_merge::tmerge;

use crate::{setting_path::SettingsPath, Settings};


use std::marker::PhantomData;
#[derive(Debug, PartialEq, Eq)]


pub struct Loaded {
    base: LoadState,
    user: LoadState,
}

impl Default for Loaded {
    fn default() -> Self {
        Loaded {
            base: LoadState::NotLoaded,
            user: LoadState::NotLoaded,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub enum SettingLoadStage {
    #[default]
    CreateBase,
    LoadStart,
    Loading(Loaded),
    LoadOver,
}

#[derive(Debug, Resource, Default)]
pub struct SettingLoadStageWrap<S>
where
    S: Settings,
{
    pub setting_load_stage: SettingLoadStage,
    _phantom_data: PhantomData<S>,
}

#[derive(Resource, Default)]
pub struct SettingsHandle<S>
where
    S: Settings,
{
    pub base_handle: Option<Handle<S>>,
    pub user_handle: Option<Handle<S>>,
}

#[allow(dead_code)]
pub(crate) fn start_create_base<S>(paths: Res<SettingsPath<S>>)
where
    S: Settings,
{
    if paths.base_config_dir.exists() {
        return;
    }

    todo!("create base settings")
}

pub(crate) fn is_load_setting<S>(stage: Res<SettingLoadStageWrap<S>>) -> bool
where
    S: Settings,
{
    SettingLoadStage::LoadStart == stage.setting_load_stage
}

pub(crate) fn startup_load_settings<S>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    paths: Res<SettingsPath<S>>,
    mut load_stage: ResMut<SettingLoadStageWrap<S>>,
) where
    S: Settings,
{
    if let SettingLoadStage::CreateBase = load_stage.setting_load_stage {
        load_stage.setting_load_stage = SettingLoadStage::Loading(Loaded::default());

        let mut handle = SettingsHandle::<S>::default();
        if paths.base_config_dir.exists() {
            handle.base_handle = Some(asset_server.load(paths.get_base_config_path()));
            if let SettingLoadStage::Loading(loaded) = &mut load_stage.setting_load_stage {
                loaded.base = LoadState::Loading;
            }
        } else if let SettingLoadStage::Loading(loaded) = &mut load_stage.setting_load_stage {
            loaded.base = LoadState::Loaded;
        }
        if paths.user_config_dir.exists() {
            handle.user_handle = Some(asset_server.load(paths.get_user_config_path()));
            if let SettingLoadStage::Loading(loaded) = &mut load_stage.setting_load_stage {
                loaded.user = LoadState::Loading;
            }
        } else if let SettingLoadStage::Loading(loaded) = &mut load_stage.setting_load_stage {
            loaded.user = LoadState::Loaded;
        }
        commands.insert_resource(handle);

        if let SettingLoadStage::Loading(loaded) = &load_stage.setting_load_stage {
            if loaded.base == LoadState::Loaded && loaded.user == LoadState::Loaded {
                load_stage.setting_load_stage = SettingLoadStage::LoadOver;
            }
        }
    }
}

pub(crate) fn refresh_final_settings<S>(
    mut asset_event_reader: EventReader<AssetEvent<S>>,
    handle: Res<SettingsHandle<S>>,
    mut load_stage: ResMut<SettingLoadStageWrap<S>>,
    assets: Res<Assets<S>>,
    mut s: ResMut<S>,
) where
    S: Settings,
{
    asset_event_reader.read().for_each(|event| match event {
        AssetEvent::Added { id } | AssetEvent::Modified { id } => {
            let mut base_asset = None;
            if let Some(base) = &handle.base_handle {
                if base.id() == *id {
                    if let Some(b) = assets.get(base) {
                        base_asset = Some(b);
                        if let SettingLoadStage::Loading(loaded) =
                            &mut load_stage.setting_load_stage
                        {
                            loaded.base = LoadState::Loaded;
                        }
                    }
                }
            }

            let mut user_asset = None;
            if let Some(user) = &handle.user_handle {
                if user.id() == *id {
                    if let Some(u) = assets.get(user) {
                        user_asset = Some(u);
                        if let SettingLoadStage::Loading(loaded) =
                            &mut load_stage.setting_load_stage
                        {
                            loaded.user = LoadState::Loaded;
                        }
                    }
                }
            }

            match (base_asset, user_asset) {
                (Some(base), Some(user)) => {
                    *s = tmerge(base, user).unwrap();
                }
                (Some(base), None) => {
                    *s = base.clone();
                }
                _ => {}
            }
        }
        _ => {}
    });

    if let SettingLoadStage::Loading(loaded) = &load_stage.setting_load_stage {
        if loaded.base == LoadState::Loaded && loaded.user == LoadState::Loaded {
            load_stage.setting_load_stage = SettingLoadStage::LoadOver;
        }
    }
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

        let merge = tmerge(a, &b).unwrap();
        assert_eq!(b, merge);
    }
}
