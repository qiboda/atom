use std::sync::Arc;

use atom_utils::input::DefaultInputMap;
use bevy::{asset::Asset, math::VectorSpace, prelude::*, reflect::Reflect};
use bevy_console::{clap::Parser, ConsoleCommand};
use bevy_tnua::{
    builtins::{TnuaBuiltinJump, TnuaBuiltinWalk},
    controller::TnuaController,
};
use leafwing_input_manager::prelude::*;
use serde::{Deserialize, Serialize};
use settings::{persist::PersistSettingEvent, setting_path::SettingsPath, Setting};

use crate::unit::Player;

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

#[derive(Debug, Serialize, Reflect, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum PlayerAction {
    Move,
    Jump,
    Ability1,
    Ability2,
    Ability3,
    Ability4,
}

impl Actionlike for PlayerAction {
    fn input_control_kind(&self) -> InputControlKind {
        match self {
            PlayerAction::Move => InputControlKind::DualAxis,
            _ => InputControlKind::Button,
        }
    }
}

impl DefaultInputMap<PlayerAction> for PlayerAction {
    fn default_input_map() -> InputMap<PlayerAction> {
        let mut input_map =
            InputMap::default().with_dual_axis(PlayerAction::Move, KeyboardVirtualDPad::WASD);

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
    player_query: Query<&GlobalTransform, With<Player>>,
    mut query: Query<&mut TnuaController>,
) {
    let player_transform = player_query.get_single().unwrap();

    let Ok(mut controller) = query.get_single_mut() else {
        return;
    };

    let Ok(player_action) = action.get_single() else {
        return;
    };

    let axis_move = player_action.clamped_axis_pair(&PlayerAction::Move);
    let velocity = Vec3::new(-axis_move.x, 0.0, axis_move.y).normalize_or_zero();
    controller.basis(TnuaBuiltinWalk {
        desired_velocity: velocity * 3.7,
        desired_forward: player_transform.forward().as_vec3(),
        float_height: 0.0,
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
