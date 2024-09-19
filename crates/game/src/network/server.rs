use bevy::{
    app::{Plugin, Startup},
    log::info,
    prelude::*,
};
use bevy_tnua::{prelude::TnuaController, TnuaUserControlsSystemSet};
use datatables::{
    tables_system_param::TableReader,
    unit::{TbPlayer, TbPlayerRow},
};
use leafwing_input_manager::prelude::ActionState;
use lightyear::{prelude::*, shared::replication::components::Controlled};
use rand::Rng;
use server::ServerCommands;

use crate::{
    input::setting::{apply_action_state_to_player_movement, PlayerAction},
    network::shared::REPLICATION_GROUP,
    scene::SceneServerPlugin,
    state::GameState,
    unit::{
        monster::ServerMonsterBundle,
        player::{BornLocation, Player, PlayerId, ServerPlayerBundle},
    },
};

pub struct GameServerPlugin;

impl Plugin for GameServerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // add our server-specific logic. Here we will just start listening for incoming connections
        app.add_plugins(SceneServerPlugin)
            .add_systems(OnEnter(GameState::InitGame), start_server)
            .add_systems(
                PreUpdate,
                // this system will replicate the inputs of a client to other clients
                // so that a client can predict other clients
                (replicate_inputs).after(MainSet::EmitEvents),
            )
            .add_systems(
                FixedUpdate,
                replicate_player_movement.in_set(TnuaUserControlsSystemSet),
            )
            .add_systems(Update, (handle_connections, handle_disconnections));
    }
}

fn start_server(mut commands: Commands) {
    commands.start_server();

    // let targets = NetworkTarget::All;
    // commands.replicate_resource::<ClientController, DefaultChannel>(targets);
    warn!("start_server");
}

#[allow(dead_code)]
fn end_server(mut commands: Commands) {
    // commands.stop_replicate_resource::<ClientController>();
    commands.stop_server()
}

/// Server connection system, create a player upon connection
fn handle_connections(
    mut connections: EventReader<server::ConnectEvent>,
    mut commands: Commands,
    player_table_reader: TableReader<TbPlayer>,
) {
    for connection in connections.read() {
        let client_id = connection.client_id;
        warn!(
            "connect server for client {:?}, is local:{}",
            client_id,
            client_id.is_local()
        );

        // server and client are running in the same app, no need to replicate to the local client
        let replicate = server::Replicate {
            sync: server::SyncTarget {
                prediction: NetworkTarget::All,
                interpolation: NetworkTarget::All,
            },
            controlled_by: server::ControlledBy {
                target: NetworkTarget::Single(client_id),
                ..default()
            },
            group: REPLICATION_GROUP,
            ..default()
        };

        let table_row_data = player_table_reader
            .get_row(&10010001)
            .expect("player table not found by id 10010001");
        let table_row = TbPlayerRow::new(10010001, Some(table_row_data.clone()));

        let location = random_location();
        info!(
            "location: {:?}, capsule radius: {}, height: {}",
            location, table_row_data.capsule_radius, table_row_data.capsule_height
        );

        commands
            .spawn((
                ServerPlayerBundle {
                    unit_bundle: default(),
                    born_location: BornLocation(location),
                    player_id: PlayerId(client_id),
                    player: crate::unit::player::Player,
                    tb_row: table_row,
                },
                replicate,
                // DisabledComponent::<Transform>::default(),
            ))
            .insert(Transform::from_translation(location));

        commands
            .spawn((
                ServerMonsterBundle {
                    unit_bundle: Default::default(),
                    monster: crate::unit::monster::Monster,
                },
                server::Replicate {
                    sync: server::SyncTarget {
                        prediction: NetworkTarget::All,
                        interpolation: NetworkTarget::All,
                    },
                    controlled_by: server::ControlledBy {
                        target: NetworkTarget::None,
                        ..default()
                    },
                    ..default()
                },
            ))
            .insert(Transform::from_translation(
                location + Vec3::new(5.0, 2.0, 0.0),
            ));
    }
}

fn random_location() -> Vec3 {
    let mut rng = rand::thread_rng();
    let x = rng.gen_range(-10.0..10.0);
    let z = rng.gen_range(-10.0..10.0);
    let y = 2.0;
    Vec3::new(x, y, z)
}

/// Handle client disconnections: we want to despawn every entity that was controlled by that client.
///
/// Lightyear creates one entity per client, which contains metadata associated with that client.
/// You can find that entity by calling `ConnectionManager::client_entity(client_id)`.
///
/// That client entity contains the `ControlledEntities` component, which is a set of entities that are controlled by that client.
///
/// By default, lightyear automatically despawns all the `ControlledEntities` when the client disconnects;
/// but in this example we will also do it manually to showcase how it can be done.
/// (however we don't actually run the system)
fn handle_disconnections(
    mut commands: Commands,
    mut disconnections: EventReader<server::DisconnectEvent>,
    manager: Res<server::ConnectionManager>,
    client_query: Query<&server::ControlledEntities>,
) {
    for disconnection in disconnections.read() {
        warn!("Client {:?} disconnected", disconnection.client_id);
        if let Ok(client_entity) = manager.client_entity(disconnection.client_id) {
            if let Ok(controlled_entities) = client_query.get(client_entity) {
                for entity in controlled_entities.entities() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

/// When we receive the input of a client, broadcast it to other clients
/// so that they can predict this client's movements accurately
fn replicate_inputs(
    mut connection: ResMut<server::ConnectionManager>,
    mut input_events: ResMut<Events<server::MessageEvent<InputMessage<PlayerAction>>>>,
) {
    for mut event in input_events.drain() {
        let client_id = *event.context();

        // Optional: do some validation on the inputs to check that there's no cheating

        // rebroadcast the input to other clients
        connection
            .send_message_to_target::<InputChannel, _>(
                &mut event.message,
                NetworkTarget::AllExceptSingle(client_id),
            )
            .unwrap()
    }
}

/// Read inputs and move players
///
/// If we didn't receive the input for a given player, we do nothing (which is the default behaviour from lightyear),
/// which means that we will be using the last known input for that player
/// (i.e. we consider that the player kept pressing the same keys).
/// see: https://github.com/cBournhonesque/lightyear/issues/492
#[allow(clippy::type_complexity)]
pub(crate) fn replicate_player_movement(
    mut query: Query<
        (
            &mut TnuaController,
            &ActionState<PlayerAction>,
            &GlobalTransform,
        ),
        // 排除掉由服务器控制的玩家
        (With<Player>, Without<Controlled>),
    >,
    // tick_manager: Res<TickManager>,
) {
    // let tick = tick_manager.tick();
    for (mut controller, player_action, global_transform) in query.iter_mut() {
        apply_action_state_to_player_movement(&mut controller, player_action, global_transform);
    }
}
