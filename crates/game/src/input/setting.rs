use std::sync::Arc;

use bevy::{
    asset::Asset,
    prelude::{EventWriter, KeyCode, Res, Resource},
    reflect::Reflect,
};
use bevy_console::{clap::Parser, ConsoleCommand};
use leafwing_input_manager::{input_map::InputMap, user_input::KeyboardVirtualDPad, Actionlike};
use serde::{Deserialize, Serialize};
use settings::{persist::PersistSettingEvent, setting_path::SettingsPath};
use settings_derive::Setting;

#[derive(Resource, Serialize, Reflect, Deserialize, Debug, Asset, Clone, Setting)]
pub struct InputSetting {
    player_input_map: InputMap<PlayerAction>,
}

impl Default for InputSetting {
    fn default() -> Self {
        Self {
            player_input_map: PlayerAction::default_input_map(),
        }
    }
}

#[derive(Debug, Actionlike, Serialize, Reflect, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum PlayerAction {
    Move,
    Jump,
    Ability1,
    Ability2,
    Ability3,
    Ability4,
}

pub trait DefaultInputMap<T: Actionlike> {
    fn default_input_map() -> InputMap<T>;
}

impl DefaultInputMap<PlayerAction> for PlayerAction {
    fn default_input_map() -> InputMap<PlayerAction> {
        let mut input_map = InputMap::new([(
            PlayerAction::Move,
            // Define a virtual D-pad using four arbitrary keys.
            // You can also use GamepadVirtualDPad to create similar ones using gamepad buttons.
            KeyboardVirtualDPad::new(
                KeyCode::ArrowUp,
                KeyCode::ArrowDown,
                KeyCode::ArrowLeft,
                KeyCode::ArrowRight,
            ),
        )]);

        input_map.insert(PlayerAction::Jump, KeyCode::Space);

        input_map.insert(PlayerAction::Ability1, KeyCode::KeyQ);
        input_map.insert(PlayerAction::Ability2, KeyCode::KeyW);
        input_map.insert(PlayerAction::Ability3, KeyCode::KeyE);
        input_map.insert(PlayerAction::Ability4, KeyCode::KeyR);

        input_map
    }
}

/// Prints given arguments to the console
#[derive(Parser, ConsoleCommand)]
#[command(name = "input.setting.persist")]
pub struct InputSettingPersistCommand;

pub fn input_setting_persist_command(
    mut persist: ConsoleCommand<InputSettingPersistCommand>,
    mut event_writer: EventWriter<PersistSettingEvent<InputSetting>>,
    input_setting: Res<InputSetting>,
    input_setting_path: Res<SettingsPath<InputSetting>>,
) {
    if let Some(Ok(InputSettingPersistCommand)) = persist.take() {
        event_writer.send(PersistSettingEvent {
            persist_path: input_setting_path.clone(),
            data: Arc::new(input_setting.clone()),
        });
    }
}
