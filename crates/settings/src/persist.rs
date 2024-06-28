use std::{path::PathBuf, sync::Arc};

use crate::{setting_path::SettingsPath, Setting};
use atom_utils::async_event::EventChannelSender;
use bevy::{prelude::*, tasks::IoTaskPool};

#[derive(Event, Default)]
pub struct PersistSettingEvent<S>
where
    S: Setting,
{
    // only directory, without file name
    pub persist_path: SettingsPath<S>,
    pub data: Arc<S>,
}

#[derive(Event, Default)]
pub struct PersistSettingEndEvent<S>
where
    S: Setting,
{
    // only directory, without file name
    pub persist_path: SettingsPath<S>,
    pub create_game_setting: bool,
    pub create_user_setting: bool,
}

pub(crate) fn persist<S>(
    mut reader: EventReader<PersistSettingEvent<S>>,
    event_channel_sender: Res<EventChannelSender<PersistSettingEndEvent<S>>>,
) where
    S: Setting,
{
    for event in reader.read() {
        let path = event.persist_path.clone();
        let data = event.data.clone();
        if let Ok(sender) = event_channel_sender.lock() {
            let sender = sender.clone();
            let thread_pool = IoTaskPool::get();
            thread_pool
                .spawn(async move {
                    fn save_setting_to_path<S: Setting>(
                        dir: &PathBuf,
                        filename: PathBuf,
                        data: &S,
                    ) {
                        if std::fs::create_dir_all(dir).is_err() {
                            error!(
                                "Couldn't create the settings directory at {:?}",
                                dir.as_os_str()
                            );
                        }

                        let settings_str = toml::to_string(data).unwrap_or_else(|_| {
                            panic!(
                                "Couldn't serialize the settings to toml {:?}",
                                dir.as_os_str()
                            )
                        });

                        std::fs::write(filename.clone(), settings_str).unwrap_or_else(
                            |_| 
                            panic!("couldn't persist the settings {:?} while trying to write the string tg disk", filename.as_os_str())
                        );
                    }

                    let mut create_user_setting = false;
                    let mut create_game_setting = false;
                    if let Some(ref user_config_dir) = path.user_config_dir {
                        save_setting_to_path(
                            user_config_dir,
                            path.get_user_config_path().unwrap(),
                            &*data,
                        );
                        create_user_setting = true;
                    }
                    if let Some(ref game_config_dir) = path.game_config_dir {
                        save_setting_to_path(
                            game_config_dir,
                            path.get_game_config_path().unwrap(),
                            &*data,
                        );
                        create_game_setting = true;
                    }

                    if let Err(e) = sender.send(PersistSettingEndEvent {
                        persist_path: path.clone(),
                        create_game_setting,
                        create_user_setting,
                    }) {
                        println!("Error sending persist end event: {:?}", e)
                    }
                })
                .detach();
        }
    }
}
