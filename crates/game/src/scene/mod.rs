use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;
use bevy_landmass::{Agent, Agent3dBundle, ArchipelagoRef3d};
use bevy_tnua_avian3d::TnuaAvian3dSensorShape;
use datatables::{
    tables_system_param::TableReader,
    unit::{TbPlayer, TbPlayerRow},
};
use lightyear::{prelude::client::Predicted, shared::replication::components::Controlled};
use oxidized_navigation::NavMeshAffector;
use terrain::TerrainState;

use crate::{
    ai::{brain::follow_player::build_ai_entity, nav::nav_move::AgentArchipelagoRef},
    input::setting::PlayerInputSetting,
    state::GameState,
    unit::{
        base::{ClientUnitBundle, UNIT_HEIGHT, UNIT_RADIUS},
        monster::{ClientMonsterBundle, Monster},
        player::{BornLocation, ClientPlayerBundle, Player, PlayerId},
        UnitPlugin,
    },
};

#[derive(Default, Debug)]
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(UnitPlugin)
            .add_systems(OnEnter(GameState::InitGame), init_scene)
            .add_systems(Update, (handle_new_monster, handle_new_player));
    }
}

fn init_scene(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_game_state: ResMut<NextState<GameState>>,
    mut terrain_state: ResMut<NextState<TerrainState>>,
) {
    // terrain_state.set(TerrainState::LoadAssets);

    let plane_3d = Plane3d::new(Vec3::Y, Vec2::new(20.0, 20.0));
    let plane_mesh = Mesh::from(plane_3d);
    let mesh = meshes.add(plane_mesh.clone());

    commands.spawn((
        Name::new("Plane"),
        MaterialMeshBundle {
            mesh,
            material: materials.add(StandardMaterial {
                base_color: Color::WHITE,
                unlit: true,
                ..Default::default()
            }),
            ..Default::default()
        },
        RigidBody::Static,
        Collider::trimesh_from_mesh(&plane_mesh).unwrap(),
        NavMeshAffector,
    ));

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 10000.0,
            shadows_enabled: true,
            shadow_depth_bias: 0.0,
            shadow_normal_bias: 0.0,
        },
        transform: Transform::from_xyz(100.0, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.insert_resource(AmbientLight {
        color: LinearRgba {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        }
        .into(),
        brightness: 1000.0,
    });

    next_game_state.set(GameState::RunGame);
    info!("init scene ok");
}

#[allow(clippy::type_complexity)]
fn handle_new_monster(
    mut commands: Commands,
    mut monster_query: Query<Entity, (With<Predicted>, Added<Monster>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    archipelago_ref: Res<AgentArchipelagoRef>,
) {
    info!("archipelago_ref: {}", archipelago_ref.archipelago_entity);
    for monster_entity in monster_query.iter_mut() {
        info!("handle new monster");
        commands
            .entity(monster_entity)
            .insert((ClientMonsterBundle {
                unit_bundle: ClientUnitBundle {
                    name: Name::new("monster"),
                    rigid_body: RigidBody::Dynamic,
                    collider: Collider::capsule(UNIT_RADIUS, UNIT_HEIGHT),
                    tuna_sensor_shape: TnuaAvian3dSensorShape(Collider::capsule(
                        UNIT_RADIUS,
                        UNIT_HEIGHT,
                    )),
                    ..Default::default()
                },
                agent_bundle: Agent3dBundle {
                    agent: Agent {
                        radius: UNIT_RADIUS,
                        height: UNIT_RADIUS * 2.0 + UNIT_HEIGHT,
                        desired_speed: 3.7,
                        max_speed: 5.0,
                    },
                    archipelago_ref: ArchipelagoRef3d::new(archipelago_ref.archipelago_entity),
                    velocity: Default::default(),
                    target: Default::default(),
                    state: Default::default(),
                    desired_velocity: Default::default(),
                },
            },))
            .insert(Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)))
            .with_children(|parent| {
                parent.spawn(MaterialMeshBundle {
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    mesh: meshes.add(Mesh::from(Capsule3d::new(UNIT_RADIUS, UNIT_HEIGHT))),
                    material: materials.add(StandardMaterial::from_color(LinearRgba::BLUE)),
                    ..Default::default()
                });
            });

        build_ai_entity(&mut commands, monster_entity);
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
            .insert((ClientPlayerBundle {
                unit_bundle: ClientUnitBundle {
                    name: Name::new("player".to_string() + player_id.0.to_string().as_str()),
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
                ..Default::default()
            },))
            .with_children(|parent| {
                parent.spawn(MaterialMeshBundle {
                    transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                    mesh: meshes.add(Mesh::from(Capsule3d::new(
                        table_row_data.capsule_radius,
                        table_row_data.capsule_height,
                    ))),
                    material: materials.add(StandardMaterial::from_color(LinearRgba::RED)),
                    ..default()
                });
            })
            .insert((Transform::from_translation(born_location.0),));

        if has_controlled {
            info!("insert player input manager");
            commands
                .entity(player_entity)
                .insert((player_input_setting.player_input_map.clone(),));
        }
    }
}
