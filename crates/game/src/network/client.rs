use std::ops::Not;

use crate::camera::GameCameraPlugin;
use crate::input::setting::{
    input_setting_persist_command, update_player_input, PlayerAction, PlayerInputSetting,
    PlayerInputSettingPersistCommand,
};
use crate::state::{next_to_init_game_state, GameState};
use crate::unit::bundle::ClientUnitBundle;
use crate::unit::player::{BornLocation, PlayerId};
use crate::unit::Player;
use atom_utils::follow::TransformFollowPlugin;
use avian3d::prelude::{Collider, RigidBody};
use bevy::winit::WinitSettings;
use bevy::{app::Plugin, prelude::*};
use bevy_atmosphere::plugin::AtmospherePlugin;
use bevy_console::AddConsoleCommand;
use bevy_tnua::controller::TnuaControllerPlugin;
use bevy_tnua::TnuaUserControlsSystemSet;
use bevy_tnua_avian3d::{TnuaAvian3dPlugin, TnuaAvian3dSensorShape};
use datatables::tables_system_param::TableReader;
use datatables::unit::{TbPlayer, TbPlayerRow};

use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use lightyear::shared::replication::components::Controlled;

use crate::scene::init_scene;

#[derive(Debug, Default)]
pub struct GameClientPlugin;

impl Plugin for GameClientPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // .add_plugins(SettingPlugin::<PlayerInputSetting> {
            //     paths: SettingsPath::default(),
            // })
            .insert_resource(WinitSettings {
                focused_mode: bevy::winit::UpdateMode::Continuous,
                unfocused_mode: bevy::winit::UpdateMode::Continuous,
            })
            .add_plugins(TransformFollowPlugin)
            .add_plugins(GameCameraPlugin)
            .insert_resource(PlayerInputSetting::default())
            // .add_plugins(TerrainSubsystemPlugin)
            .add_plugins(AtmospherePlugin)
            .add_plugins((
                TnuaControllerPlugin::new(FixedUpdate),
                TnuaAvian3dPlugin::new(FixedUpdate),
            ))
            .insert_state(GameState::default())
            .add_console_command::<PlayerInputSettingPersistCommand, _>(
                input_setting_persist_command,
            )
            .add_systems(Startup, client_startup)
            .add_systems(Last, client_shutdown)
            .add_systems(
                Update,
                (handle_client_connect, handle_client_disconnect)
                    .after(MainSet::Receive)
                    .before(PredictionSet::SpawnPrediction),
            )
            .add_systems(OnEnter(GameState::InitGame), init_scene)
            .add_systems(Update, handle_new_player)
            .add_systems(
                FixedUpdate,
                update_player_input
                    .in_set(TnuaUserControlsSystemSet)
                    .run_if(in_state(GameState::RunGame)),
            )
            .add_systems(Last, next_to_init_game_state);
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

// controlled: 客户端控制的和host server的local client控制的会有这个组件
#[allow(clippy::type_complexity)]
fn handle_new_player(
    mut commands: Commands,
    mut player_query: Query<
        (
            Entity,
            &PlayerId,
            &mut TbPlayerRow,
            &BornLocation,
            Has<Controlled>,
        ),
        (With<Predicted>, Added<Player>),
    >,
    player_table_reader: TableReader<TbPlayer>,
    player_input_setting: Res<PlayerInputSetting>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (player_entity, player_id, mut player_row, born_location, has_controlled) in
        player_query.iter_mut()
    {
        info!("handle new player: {}", player_entity);
        let table_row_data = player_table_reader
            .get_row(player_row.key())
            .unwrap_or_else(|| panic!("player table row not found by id {:?}", player_row.key()));
        player_row.set_data(Some(table_row_data.clone()));

        commands
            .entity(player_entity)
            .insert((
                ClientUnitBundle {
                    name: Name::new("player".to_string() + player_id.0.to_string().as_str()),
                    mesh: meshes.add(Mesh::from(Capsule3d::new(
                        table_row_data.capsule_radius,
                        table_row_data.capsule_height,
                    ))),
                    material: materials.add(StandardMaterial::from_color(LinearRgba::RED)),
                    rigid_body: RigidBody::Dynamic,
                    collider: Collider::capsule(
                        table_row_data.capsule_radius,
                        table_row_data.capsule_height,
                    ),
                    tuna_sensor_shape: TnuaAvian3dSensorShape(Collider::capsule(
                        table_row_data.capsule_radius,
                        table_row_data.capsule_height,
                    )),
                    ..Default::default()
                },
                ActionState::<PlayerAction>::default(),
            ))
            .insert((
                Visibility::Visible,
                Transform::from_translation(born_location.0),
            ));

        if has_controlled {
            info!("insert player input manager");
            commands
                .entity(player_entity)
                .insert((player_input_setting.player_input_map.clone(),));
        }
    }
}
