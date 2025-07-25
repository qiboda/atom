use std::sync::Arc;

use atom_utils::input::DefaultInputMap;
use bevy::{asset::Asset, prelude::*, reflect::Reflect};
use bevy_tnua::{
    builtins::{TnuaBuiltinJump, TnuaBuiltinWalk},
    controller::TnuaController,
    TnuaUserControlsSystemSet,
};
use leafwing_input_manager::prelude::*;
use lightyear::prelude::client::Predicted;
use serde::{Deserialize, Serialize};
use settings::{persist::PersistSettingEvent, setting_path::SettingsPath, Setting};

use crate::{state::GameState, unit::player::Player};

#[derive(Debug, Default)]
pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app
            // .add_plugins(SettingPlugin::<PlayerInputSetting> {
            //     paths: SettingsPath::default(),
            // })
            .insert_resource(PlayerInputSetting::default())
            .add_systems(
                FixedUpdate,
                update_player_input
                    .in_set(TnuaUserControlsSystemSet)
                    .run_if(in_state(GameState::RunGame)),
            );
        // .add_console_command::<PlayerInputSettingPersistCommand, _>(
        //     input_setting_persist_command,
        // );
    }
}

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

#[derive(Debug, Serialize, Reflect, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
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
            InputMap::default().with_dual_axis(PlayerAction::Move, VirtualDPad::wasd());

        input_map.insert(PlayerAction::Jump, KeyCode::Space);

        // input_map.insert(PlayerAction::Ability1, KeyCode::KeyQ);
        // input_map.insert(PlayerAction::Ability2, KeyCode::KeyW);
        // input_map.insert(PlayerAction::Ability3, KeyCode::KeyE);
        // input_map.insert(PlayerAction::Ability4, KeyCode::KeyR);

        input_map
    }
}

/// TODO: 以后支持泛型，由于clap的Parser不支持泛型，所以暂时只能用这种方式。
// #[derive(Parser, ConsoleCommand)]
// #[command(name = "input.setting.persist")]
// pub struct PlayerInputSettingPersistCommand;

// pub fn input_setting_persist_command(
//     mut persist: ConsoleCommand<PlayerInputSettingPersistCommand>,
//     mut event_writer: EventWriter<PersistSettingEvent<PlayerInputSetting>>,
//     input_setting: Res<PlayerInputSetting>,
//     input_setting_path: Res<SettingsPath<PlayerInputSetting>>,
// ) {
//     if let Some(Ok(_cmd)) = persist.take() {
//         event_writer.send(PersistSettingEvent {
//             persist_path: input_setting_path.clone(),
//             data: Arc::new(input_setting.clone()),
//         });

//         persist.ok();
//     }
// }

#[allow(clippy::type_complexity)]
pub fn update_player_input(
    mut player_query: Query<
        (
            &mut TnuaController,
            &ActionState<PlayerAction>,
            &GlobalTransform,
        ),
        (With<Predicted>, With<Player>),
    >,
) {
    for (mut controller, player_action, player_transform) in player_query.iter_mut() {
        apply_action_state_to_player_movement(&mut controller, player_action, player_transform);
    }
}

pub(crate) fn apply_action_state_to_player_movement(
    controller: &mut TnuaController,
    player_action: &ActionState<PlayerAction>,
    player_transform: &GlobalTransform,
) {
    let axis_move = player_action.clamped_axis_pair(&PlayerAction::Move);

    let velocity = Vec3::new(-axis_move.x, 0.0, axis_move.y).normalize_or_zero();
    controller.basis(TnuaBuiltinWalk {
        desired_velocity: velocity * 3.7,
        desired_forward: Some(player_transform.forward()),
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
