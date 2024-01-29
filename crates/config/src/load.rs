use bevy::prelude::*;

use crate::{setting_path::SettingsPath, Settings};

use std::marker::PhantomData;

#[derive(Reflect, Debug, Resource)]
pub enum SettingLoadStage<S>
where
    S: Settings,
{
    CreateBase(PhantomData<S>),
    Loading(PhantomData<S>),
    LoadOver(PhantomData<S>),
}

impl<S> Default for SettingLoadStage<S>
where
    S: Settings,
{
    fn default() -> Self {
        SettingLoadStage::CreateBase(PhantomData::<S>)
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

pub(crate) fn start_create_base<S>(paths: Res<SettingsPath<S>>)
where
    S: Settings,
{
    if paths.base_config_dir.exists() {
        return;
    }

    todo!("create base settings")
}

pub(crate) fn startup_load_settings<S>(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    paths: Res<SettingsPath<S>>,
) where
    S: Settings,
{
    let mut handle = SettingsHandle::<S>::default();
    handle.base_handle = asset_server.load(paths.get_base_config_path());
    handle.user_handle = asset_server.load(paths.get_user_config_path());
    commands.insert_resource(handle);
}

pub(crate) fn refresh_final_settings<S>(
    mut asset_server: ResMut<AssetServer>,
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
