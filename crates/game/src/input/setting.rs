use std::sync::Arc;

use atom_utils::input::DefaultInputMap;
use bevy::{asset::Asset, prelude::*, reflect::Reflect};
use bevy_console::{clap::Parser, ConsoleCommand};
use bevy_tnua::{
    builtins::{TnuaBuiltinJump, TnuaBuiltinWalk},
    controller::TnuaController,
};
use leafwing_input_manager::{
    action_state::ActionState, input_map::InputMap, user_input::KeyboardVirtualDPad, Actionlike,
};
use serde::{Deserialize, Serialize};
use settings::{persist::PersistSettingEvent, setting_path::SettingsPath};
use settings_derive::Setting;

#[derive(Resource, Serialize, Reflect, Deserialize, Debug, Asset, Clone, Setting)]
pub struct PlayerInputSetting {
    pub player_input_map: InputMap<PlayerAction>,
}

impl Default for PlayerInputSetting {
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

impl DefaultInputMap<PlayerAction> for PlayerAction {
    fn default_input_map() -> InputMap<PlayerAction> {
        let mut input_map = InputMap::new([(
            PlayerAction::Move,
            // Define a virtual D-pad using four arbitrary keys.
            // You can also use GamepadVirtualDPad to create similar ones using gamepad buttons.
            KeyboardVirtualDPad::new(KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD),
        )]);

        input_map.insert(PlayerAction::Jump, KeyCode::Space);

        // input_map.insert(PlayerAction::Ability1, KeyCode::KeyQ);
        // input_map.insert(PlayerAction::Ability2, KeyCode::KeyW);
        // input_map.insert(PlayerAction::Ability3, KeyCode::KeyE);
        // input_map.insert(PlayerAction::Ability4, KeyCode::KeyR);

        input_map
    }
}

/// TODO: 以后支持泛型，由于clap的Parser不支持泛型，所以暂时只能用这种方式。
#[derive(Parser, ConsoleCommand)]
#[command(name = "input.setting.persist")]
pub struct PlayerInputSettingPersistCommand;

pub fn input_setting_persist_command(
    mut persist: ConsoleCommand<PlayerInputSettingPersistCommand>,
    mut event_writer: EventWriter<PersistSettingEvent<PlayerInputSetting>>,
    input_setting: Res<PlayerInputSetting>,
    input_setting_path: Res<SettingsPath<PlayerInputSetting>>,
) {
    if let Some(Ok(_cmd)) = persist.take() {
        event_writer.send(PersistSettingEvent {
            persist_path: input_setting_path.clone(),
            data: Arc::new(input_setting.clone()),
        });

        persist.ok();
    }
}

pub fn update_player_input(
    action: Query<&ActionState<PlayerAction>>,
    mut query: Query<&mut TnuaController>,
) {
    let Ok(mut controller) = query.get_single_mut() else {
        return;
    };

    let Ok(player_action) = action.get_single() else {
        return;
    };

    let velocity = player_action
        .clamped_axis_pair(&PlayerAction::Move)
        .map(|direction| Vec3::new(direction.x(), 0.0, direction.y()));

    controller.basis(TnuaBuiltinWalk {
        desired_velocity: velocity.unwrap() * 3.7,
        float_height: 1.5,
        ..Default::default()
    });

    if player_action.pressed(&PlayerAction::Jump) {
        controller.action(TnuaBuiltinJump {
            // The height is the only mandatory field of the jump button.
            height: 2.0,
            // `TnuaBuiltinJump` also has customization fields with sensible defaults.
            ..Default::default()
        });
    }
}
