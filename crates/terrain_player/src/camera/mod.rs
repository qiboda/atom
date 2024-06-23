// use bevy::math::vec2;
// use bevy::prelude::*;

// use smooth_bevy_cameras::controllers::fps::{
//     FpsCameraBundle, FpsCameraController, FpsCameraPlugin,
// };
// use smooth_bevy_cameras::LookTransformPlugin;

// pub struct SmoothCameraPlugin;

// impl Plugin for SmoothCameraPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_plugins((
//             LookTransformPlugin,
//             FpsCameraPlugin {
//                 override_input_system: false,
//             },
//         ))
//         .add_systems(Startup, setup);
//     }
// }

// fn setup(mut commands: Commands) {
//     commands
//         .spawn(FpsCameraBundle::new(
//             FpsCameraController {
//                 enabled: true,
//                 mouse_rotate_sensitivity: vec2(0.3, 0.3),
//                 translate_sensitivity: 10.0,
//                 smoothing_weight: 0.9,
//             },
//             Vec3::new(0.0, 10.0, 10.0),
//             Vec3::ZERO,
//             Vec3::Y,
//         ))
//         .insert(Camera3dBundle {
//             projection: Projection::Perspective(PerspectiveProjection {
//                 far: 10000.0,
//                 ..Default::default()
//             }),
//             ..default()
//         });
// }
