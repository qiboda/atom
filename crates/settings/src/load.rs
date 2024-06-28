use bevy::{asset::LoadState, prelude::*};
use serde_merge::omerge;

use crate::{
    persist::{PersistSettingEndEvent, PersistSettingEvent},
    setting_path::SettingsPath,
    Setting,
};

use std::{marker::PhantomData, ops::Not, sync::Arc};

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
            if event.create_game_setting || event.create_user_setting {
                stage.setting_load_stage = SettingLoadStage::LoadStart;
            }
        }
    }
}

// 创建文件，并加载到文件中。
// 加载文件。
pub(crate) fn create_game_setting<S>(
    paths: Res<SettingsPath<S>>,
    mut persist_event: EventWriter<PersistSettingEvent<S>>,
    mut load_stage: ResMut<SettingLoadStageWrap<S>>,
    s: Res<S>,
) where
    S: Setting,
{
    if let SettingLoadStage::LoadStart = load_stage.setting_load_stage {
        if let Some(game_config_path) = paths.get_game_config_path() {
            if game_config_path.exists().not() {
                let event = PersistSettingEvent {
                    persist_path: paths.clone(),
                    data: Arc::new(s.clone()),
                };
                persist_event.send(event);
            } else {
                load_stage.setting_load_stage = SettingLoadStage::LoadStart;
            }
        }
    }
}

pub(crate) fn start_load_settings<S>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    paths: Res<SettingsPath<S>>,
    mut load_stage: ResMut<SettingLoadStageWrap<S>>,
) where
    S: Setting,
{
    if let SettingLoadStage::LoadStart = load_stage.setting_load_stage {
        load_stage.setting_load_stage = SettingLoadStage::Loading(SettingLoadState::default());

        let mut handle = InnerSettingHandle::<S>::default();

        if let Some(game_path) = paths.get_game_config_path() {
            if game_path.exists() {
                handle.game_handle = Some(asset_server.load(paths.get_game_config_path().unwrap()));
                if let SettingLoadStage::Loading(loaded) = &mut load_stage.setting_load_stage {
                    loaded.game = LoadState::Loading;
                }
            } else if let SettingLoadStage::Loading(loaded) = &mut load_stage.setting_load_stage {
                loaded.game = LoadState::Loaded;
            }
        }
        if let Some(user_path) = paths.get_user_config_path() {
            if user_path.exists() {
                handle.user_handle = Some(asset_server.load(paths.get_user_config_path().unwrap()));
                if let SettingLoadStage::Loading(loaded) = &mut load_stage.setting_load_stage {
                    loaded.user = LoadState::Loading;
                }
            } else if let SettingLoadStage::Loading(loaded) = &mut load_stage.setting_load_stage {
                loaded.user = LoadState::Loaded;
            }
        }

        commands.insert_resource(handle);

        if let SettingLoadStage::Loading(loaded) = &load_stage.setting_load_stage {
            if loaded.game == LoadState::Loaded && loaded.user == LoadState::Loaded {
                load_stage.setting_load_stage = SettingLoadStage::LoadOver;
            }
        }
    }
}

pub(crate) fn refresh_final_settings<S>(
    mut asset_event_reader: EventReader<AssetEvent<S>>,
    handle: Res<InnerSettingHandle<S>>,
    mut load_stage: ResMut<SettingLoadStageWrap<S>>,
    assets: Res<Assets<S>>,
    mut setting_update_event: EventWriter<SettingUpdateEvent<S>>,
    mut s: ResMut<S>,
) where
    S: Setting,
{
    asset_event_reader.read().for_each(|event| match event {
        AssetEvent::Added { id } | AssetEvent::Modified { id } => {
            let mut game_asset = None;
            if let Some(game) = &handle.game_handle {
                if game.id() == *id {
                    if let Some(b) = assets.get(game) {
                        game_asset = Some(b);
                        if let SettingLoadStage::Loading(loaded) =
                            &mut load_stage.setting_load_stage
                        {
                            loaded.game = LoadState::Loaded;
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

            match (game_asset, user_asset) {
                (Some(game), Some(user)) => {
                    *s = omerge(game, user).unwrap();
                    setting_update_event.send_default();
                }
                (Some(game), None) => {
                    *s = game.clone();
                    setting_update_event.send_default();
                }
                _ => {}
            }
        }
        _ => {}
    });

    if let SettingLoadStage::Loading(loaded) = &load_stage.setting_load_stage {
        if loaded.game == LoadState::Loaded && loaded.user == LoadState::Loaded {
            load_stage.setting_load_stage = SettingLoadStage::LoadOver;
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
