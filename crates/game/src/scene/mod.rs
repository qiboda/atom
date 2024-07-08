use aery::edges::RelationCommands;
use atom_camera::{setting::CameraSetting, CameraTracker};
use atom_utils::{
    follow::{Following, RelativeTransform, RelativeTransformFreedom},
    transform::{
        RotationLockedFreedom, ScaleLockedFreedom, TransformFreedom, TransformLockedFreedom,
        TranslationLockedFreedom,
    },
};
use avian3d::{collision::Collider, prelude::RigidBody};
use bevy::prelude::*;
use leafwing_input_manager::InputManagerBundle;

use crate::{
    input::setting::PlayerInputSetting,
    state::GameState,
    unit::{bundle::UnitBundle, Player},
};

pub fn init_scene(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    camera_setting: Res<CameraSetting>,
    player_input_setting: Res<PlayerInputSetting>,
    mut camera_tracking: ResMut<CameraTracker>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::new(0.0, -100.0, 100.0)),
        ..Default::default()
    });

    let plane = Plane3d {
        normal: Dir3::Y,
        half_size: Vec2::splat(100.0),
    };
    let mesh = meshes.add(plane);

    commands
        .spawn(MaterialMeshBundle {
            mesh,
            material: materials.add(StandardMaterial::from_color(LinearRgba::gray(0.3))),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ..Default::default()
        })
        .insert((
            RigidBody::Static,
            Collider::trimesh_from_mesh(&plane.into()).unwrap(),
        ));

    let player_entity = commands
        .spawn((
            UnitBundle {
                name: Name::new("player".to_string()),
                mat_mesh: MaterialMeshBundle {
                    mesh: meshes.add(Mesh::from(Capsule3d::new(0.5, 2.0))),
                    material: materials.add(StandardMaterial::from_color(LinearRgba::RED)),
                    transform: Transform::from_translation(Vec3::new(0.0, 2.0, 0.0)),
                    ..Default::default()
                },
                ..Default::default()
            },
            Player,
        ))
        .insert(InputManagerBundle::with_map(
            player_input_setting.player_input_map.clone(),
        ))
        .id();

    let camera_entity = commands
        .spawn(Camera3dBundle {
            transform: Transform::from_translation(Vec3::new(10.0, 10.0, 10.0))
                .looking_at(Vec3::ZERO, Dir3::Y),
            ..Default::default()
        })
        .insert((
            RelativeTransform(
                Transform::from_translation(Vec3::new(10.0, 10.0, 10.0))
                    .looking_at(Vec3::ZERO, Dir3::Y),
            ),
            RelativeTransformFreedom(TransformFreedom::Lock(TransformLockedFreedom {
                locked_translation: Some(TranslationLockedFreedom {
                    locked_x: false,
                    locked_y: true,
                    locked_z: false,
                }),
                locked_rotation: Some(RotationLockedFreedom {
                    locked_pitch: true,
                    locked_yaw: true,
                    locked_roll: true,
                }),
                locked_scale: Some(ScaleLockedFreedom {
                    locked_x: true,
                    locked_y: true,
                    locked_z: true,
                }),
            })),
        ))
        .set::<Following>(player_entity)
        .id();

    // 保存在全局，
    commands.spawn(InputManagerBundle::with_map(
        camera_setting.camera_input_map.clone(),
    ));

    camera_tracking.set_main_camera(camera_entity);
    info!("init scene done.");

    next_game_state.set(GameState::RunGame);
}
