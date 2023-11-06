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
    mut query: Query<(&GlobalTransform, &mut Transform), With<Camera3d>>,
) {
    const MOVE_SPEED: f32 = 10.0;
    const ROTATION_SPEED: f32 = 0.5;
    for (global_transform, mut trans) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::W) {
            trans.translation += MOVE_SPEED * global_transform.forward() * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::S) {
            trans.translation += MOVE_SPEED * global_transform.back() * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::A) {
            trans.translation += MOVE_SPEED * global_transform.left() * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::D) {
            trans.translation += MOVE_SPEED * global_transform.right() * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            trans.translation += MOVE_SPEED * Vec3::new(0.0, -1.0, 0.0) * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::ShiftRight) {
            trans.translation += MOVE_SPEED * Vec3::new(0.0, 1.0, 0.0) * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::Q) {
            trans.rotate_y(-10.0 * time.delta_seconds() * ROTATION_SPEED);
        }

        if keyboard_input.pressed(KeyCode::E) {
            trans.rotate_y(10.0 * time.delta_seconds() * ROTATION_SPEED);
        }

        if keyboard_input.pressed(KeyCode::ControlLeft) {
            trans.rotate_z(-10.0 * time.delta_seconds() * ROTATION_SPEED);
        }

        if keyboard_input.pressed(KeyCode::ControlRight) {
            trans.rotate_z(10.0 * time.delta_seconds() * ROTATION_SPEED);
        }
    }
}
