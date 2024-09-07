use atom_camera::CameraManagerPlugin;
use atom_internal::plugins::AtomDefaultPlugins;
use atom_utils::follow::TransformFollowPlugin;
use avian3d::{debug_render::PhysicsDebugPlugin, PhysicsPlugins};
use bevy::{app::Plugin, prelude::*};
use bevy_atmosphere::plugin::AtmospherePlugin;
use bevy_console::AddConsoleCommand;
use bevy_tnua::controller::TnuaControllerPlugin;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;
use input::setting::{
    input_setting_persist_command, update_player_input, PlayerAction, PlayerInputSetting,
    PlayerInputSettingPersistCommand,
};
use leafwing_input_manager::plugin::InputManagerPlugin;
use scene::init_scene;
use settings::{setting_path::SettingsPath, SettingPlugin};
use state::{next_to_init_game_state, GameState};
use terrain::TerrainSubsystemPlugin;

pub mod ai;
pub mod damage;
pub mod input;
pub mod items;
pub mod projectile;
pub mod scene;
pub mod state;
pub mod unit;

#[derive(Debug, Default)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(AtomDefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // uncomment for unthrottled FPS
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        // .add_plugins(SettingPlugin::<PlayerInputSetting> {
        //     paths: SettingsPath::default(),
        // })
        .insert_resource(PlayerInputSetting::default())
        .add_plugins(InputManagerPlugin::<PlayerAction>::default())
        .add_plugins(TransformFollowPlugin)
        .add_plugins(TerrainSubsystemPlugin)
        .add_plugins(CameraManagerPlugin)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(AtmospherePlugin)
        .add_plugins((
            TnuaControllerPlugin::default(),
            TnuaAvian3dPlugin::default(),
        ))
        .insert_state(GameState::default())
        .add_console_command::<PlayerInputSettingPersistCommand, _>(input_setting_persist_command)
        .add_systems(OnEnter(GameState::InitGame), init_scene)
        .add_systems(
            Update,
            update_player_input.run_if(in_state(GameState::RunGame)),
        )
        .add_systems(Last, next_to_init_game_state);
    }
}

// todo: add tune.
