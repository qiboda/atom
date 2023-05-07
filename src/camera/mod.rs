use bevy::prelude::*;

#[derive(Default)]
pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, camera_movement);
    }
}

fn camera_movement(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    const MOVE_SPEED: f32 = 10.0;
    for mut trans in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::W) {
            trans.translation += MOVE_SPEED * Vec3::new(1.0, 0.0, 0.0) * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::S) {
            trans.translation += MOVE_SPEED * Vec3::new(-1.0, 0.0, 0.0) * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::A) {
            trans.translation += MOVE_SPEED * Vec3::new(0.0, 0.0, -1.0) * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::D) {
            trans.translation += MOVE_SPEED * Vec3::new(0.0, 0.0, 1.0) * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::LShift) {
            trans.translation += MOVE_SPEED * Vec3::new(0.0, -1.0, 0.0) * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::RShift) {
            trans.translation += MOVE_SPEED * Vec3::new(0.0, 1.0, 0.0) * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::Q) {
            trans.rotate_around(
                Vec3::ZERO,
                Quat::from_rotation_y(-1.0 * time.delta_seconds()),
            );
        }

        if keyboard_input.pressed(KeyCode::E) {
            trans.rotate_around(
                Vec3::ZERO,
                Quat::from_rotation_y(1.0 * time.delta_seconds()),
            );
        }
    }
}
