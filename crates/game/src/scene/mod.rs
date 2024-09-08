use avian3d::prelude::{Collider, RigidBody};
use bevy::prelude::*;

use crate::state::GameState;

pub fn init_scene(
    mut commands: Commands,
    // asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    let plane_3d = Plane3d::new(Vec3::Y, Vec2::new(10.0, 10.0));
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

    // commands.spawn(Camera2dBundle::default());

    next_game_state.set(GameState::RunGame);
    info!("init scene ok");
}
