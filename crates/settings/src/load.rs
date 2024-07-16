use bevy::{
    asset::{AssetPath, LoadState},
    prelude::*,
    utils::info,
};
use serde_merge::omerge;

use crate::{
    persist::{PersistSettingEndEvent, PersistSettingEvent},
    setting_path::SettingsPath,
    Setting, SettingsLoadStatus, SettingsSource,
};

use std::{any::TypeId, fmt::Debug, marker::PhantomData, ops::Not, sync::Arc};

/// first loaded or hot loading will send this event.
#[derive(Debug, Event, Default)]
pub struct SettingUpdateEvent<S>
where
    S: Setting,
{
    phantom: PhantomData<S>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SettingLoadState {
    game: LoadState,
    user: LoadState,
}

impl Default for SettingLoadState {
    fn default() -> Self {
        SettingLoadState {
            game: LoadState::NotLoaded,
            user: LoadState::NotLoaded,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Default)]
pub enum SettingLoadStage {
    #[default]
    LoadWait,
    LoadStart,
    Loading(SettingLoadState),
    LoadOver,
}

#[derive(Debug, Resource, Default)]
pub struct SettingLoadStageWrap<S>
where
    S: Setting,
{
    pub setting_load_stage: SettingLoadStage,
    _phantom_data: PhantomData<S>,
}

#[derive(Resource, Default)]
pub(crate) struct InnerSettingHandle<S>
where
    S: Setting,
{
    pub game_handle: Option<Handle<S>>,
    pub user_handle: Option<Handle<S>>,
}

pub(crate) fn handle_persist_setting_end_event<S>(
    mut events: EventReader<PersistSettingEndEvent<S>>,
    mut stage: ResMut<SettingLoadStageWrap<S>>,
) where
    S: Setting,
{
    if stage.setting_load_stage == SettingLoadStage::LoadWait {
        for event in events.read() {
            if event.create_game_setting && event.create_user_setting {
                stage.setting_load_stage = SettingLoadStage::LoadStart;
            }
        }
    }
}

// 创建文件，并加载到文件中。
// 加载文件。
pub(crate) fn create_game_setting<S>(
    paths: Res<SettingsPath<S>>,
    settings_source: Res<SettingsSource>,
    mut persist_event: EventWriter<PersistSettingEvent<S>>,
    mut load_stage: ResMut<SettingLoadStageWrap<S>>,
    s: Res<S>,
    mut settings_status: ResMut<SettingsLoadStatus>,
) where
    S: Setting + Debug,
{
    if let SettingLoadStage::LoadWait = load_stage.setting_load_stage {
        settings_status.status.insert(TypeId::of::<S>(), false);
        if let Some(game_config_path) = paths.get_game_config_path() {
            if settings_source
                .game_source_path
                .join(game_config_path)
                .exists()
                .not()
            {
                let event = PersistSettingEvent {
                    persist_path: paths.clone(),
                    data: Arc::new(s.clone()),
                };
                persist_event.send(event);
                info!("create setting file: {:?}", *paths);
            } else {
                load_stage.setting_load_stage = SettingLoadStage::LoadStart;
            }
        }
        if let Some(user_config_path) = paths.get_user_config_path() {
            if settings_source
                .user_source_path
                .join(user_config_path)
                .exists()
                .not()
            {
                let event = PersistSettingEvent {
                    persist_path: paths.clone(),
                    data: Arc::new(s.clone()),
                };
                persist_event.send(event);
                info!("create setting file: {:?}", *paths);
            } else {
                load_stage.setting_load_stage = SettingLoadStage::LoadStart;
            }
        }
    }
}

pub(crate) fn start_load_settings<S>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    settings_source: Res<SettingsSource>,
    paths: Res<SettingsPath<S>>,
    mut load_stage: ResMut<SettingLoadStageWrap<S>>,
) where
    S: Setting + Debug,
{
    if let SettingLoadStage::LoadStart = load_stage.setting_load_stage {
        load_stage.setting_load_stage = SettingLoadStage::Loading(SettingLoadState::default());

        let mut handle = InnerSettingHandle::<S>::default();

        if let Some(game_path) = paths.get_game_config_path() {
            info!("load game setting file: {:?}", game_path);
            let asset_path = AssetPath::from(paths.get_game_config_path().unwrap())
                .with_source(settings_source.game_source_id.clone());
            handle.game_handle = Some(asset_server.load(asset_path));
            if let SettingLoadStage::Loading(loaded) = &mut load_stage.setting_load_stage {
                loaded.game = LoadState::Loading;
            }
        }
        if let Some(user_path) = paths.get_user_config_path() {
            info!("load user setting file: {:?}", user_path);
            let asset_path = AssetPath::from(paths.get_user_config_path().unwrap())
                .with_source(settings_source.user_source_id.clone());
            handle.user_handle = Some(asset_server.load(asset_path));
            if let SettingLoadStage::Loading(loaded) = &mut load_stage.setting_load_stage {
                loaded.user = LoadState::Loading;
            }
        }

        commands.insert_resource(handle);
    }
}

pub(crate) fn refresh_final_settings<S>(
    mut asset_event_reader: EventReader<AssetEvent<S>>,
    handle: Res<InnerSettingHandle<S>>,
    mut load_stage: ResMut<SettingLoadStageWrap<S>>,
    assets: Res<Assets<S>>,
    mut setting_update_event: EventWriter<SettingUpdateEvent<S>>,
    mut s: ResMut<S>,
    mut settings_status: ResMut<SettingsLoadStatus>,
) where
    S: Setting,
{
    asset_event_reader.read().for_each(|event| match event {
        AssetEvent::Added { id } | AssetEvent::Modified { id } => {
            if let Some(game) = &handle.game_handle {
                if game.id() == *id {
                    if let SettingLoadStage::Loading(loaded) = &mut load_stage.setting_load_stage {
                        loaded.game = LoadState::Loaded;
                    }
                }
            }

            if let Some(user) = &handle.user_handle {
                if user.id() == *id {
                    if let SettingLoadStage::Loading(loaded) = &mut load_stage.setting_load_stage {
                        loaded.user = LoadState::Loaded;
                    }
                }
            }
        }
        _ => {}
    });

    if let SettingLoadStage::Loading(loaded) = &load_stage.setting_load_stage {
        if loaded.game == LoadState::Loaded && loaded.user == LoadState::Loaded {
            let game_asset = handle
                .game_handle
                .as_ref()
                .and_then(|game| assets.get(game));
            let user_asset = handle
                .user_handle
                .as_ref()
                .and_then(|user| assets.get(user));
            if let (Some(game), Some(user)) = (game_asset, user_asset) {
                *s = omerge(game, user).unwrap();
                setting_update_event.send_default();
            }

            load_stage.setting_load_stage = SettingLoadStage::LoadOver;

            let status = settings_status
                .status
                .get_mut(&TypeId::of::<S>())
                .expect("must have");
            *status = true;

            info!("setting load over: {:?}", *s);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::{asset::Asset, reflect::TypePath};
    use serde::{Deserialize, Serialize};
    use serde_merge::{omerge, tmerge};

    #[derive(Serialize, Deserialize, PartialEq, Debug, Asset, TypePath)]
    struct TestSettings {
        a: Option<u32>,
        b: Option<String>,
    }

    #[test]
    fn test_mmerge() {
        let a = TestSettings {
            a: Some(1),
            b: Some("a".to_string()),
        };

        let b = TestSettings {
            a: None,
            b: Some("b".to_string()),
        };

        let merge = tmerge(a, b).unwrap();
        assert_eq!(
            TestSettings {
                a: None,
                b: Some("b".to_string()),
            },
            merge
        );
    }

    #[test]
    fn test_omerge() {
        let a = TestSettings {
            a: Some(1),
            b: Some("a".to_string()),
        };

        let b = TestSettings {
            a: None,
            b: Some("b".to_string()),
        };

        let merge = omerge(a, b).unwrap();
        assert_eq!(
            TestSettings {
                a: Some(1),
                b: Some("b".to_string()),
            },
            merge
        );
    }
}
