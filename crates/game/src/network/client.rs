use std::ops::Not;

use crate::camera::GameCameraPlugin;
use crate::input::setting::PlayerInputPlugin;
use crate::scene::SceneClientPlugin;
use crate::state::{GameState, GameStatePlugin};
use atom_utils::follow::TransformFollowPlugin;
use avian3d::prelude::PhysicsDebugPlugin;
use bevy::winit::WinitSettings;
use bevy::{app::Plugin, prelude::*};
use bevy_atmosphere::plugin::AtmospherePlugin;
use bevy_tnua::controller::TnuaControllerPlugin;
use bevy_tnua_avian3d::TnuaAvian3dPlugin;

use lightyear::prelude::client::*;
use lightyear::prelude::*;
use terrain::TerrainSubsystemPlugin;

#[derive(Debug, Default)]
pub struct GameClientPlugin;

impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(WinitSettings {
            focused_mode: bevy::winit::UpdateMode::Continuous,
            unfocused_mode: bevy::winit::UpdateMode::Continuous,
        })
        .add_plugins(SceneClientPlugin)
        .add_plugins(GameStatePlugin)
        .add_plugins(TerrainSubsystemPlugin)
        .add_plugins(AtmospherePlugin)
        .add_plugins((
            TnuaControllerPlugin::new(FixedUpdate),
            TnuaAvian3dPlugin::new(FixedUpdate),
        ))
        .add_plugins(TransformFollowPlugin)
        .add_plugins(GameCameraPlugin)
        .add_plugins(PlayerInputPlugin)
        .add_plugins(PhysicsDebugPlugin::new(FixedUpdate))
        .add_systems(OnEnter(GameState::InitGame), client_startup)
        .add_systems(Last, client_shutdown)
        .add_systems(
            Update,
            (handle_client_connect, handle_client_disconnect)
                .after(MainSet::Receive)
                .before(PredictionSet::SpawnPrediction),
        );
    }
}

fn client_startup(
    mut commands: Commands,
    network_identity: NetworkIdentity,
    networking_state: Res<State<client::NetworkingState>>,
) {
    if network_identity.is_server() {
        return;
    }

    match networking_state.get() {
        client::NetworkingState::Connected => {}
        client::NetworkingState::Connecting => {}
        client::NetworkingState::Disconnected => {
            info!("client startup");
            commands.connect_client();
        }
    }
}

fn handle_client_connect(mut client_connect_event: EventReader<client::ConnectEvent>) {
    for event in client_connect_event.read() {
        info!("client connect: {:?}", event.client_id());
        // commands.insert_resource(event.client_id());
    }
}

fn handle_client_disconnect(mut client_disconnect_event: EventReader<client::DisconnectEvent>) {
    for event in client_disconnect_event.read() {
        info!("client disconnect: {:?}", event.reason);
    }
}

fn client_shutdown(
    mut commands: Commands,
    networking_state: Res<State<client::NetworkingState>>,
    network_identity: NetworkIdentity,
    app_exit_events: EventReader<AppExit>,
) {
    if app_exit_events.is_empty().not() {
        if network_identity.is_server() {
            return;
        }
        match networking_state.get() {
            client::NetworkingState::Connected => {
                info!("client shutdown");
                commands.disconnect_client();
            }
            client::NetworkingState::Connecting => {}
            client::NetworkingState::Disconnected => {}
        }
    }
}
