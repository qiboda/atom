use aery::prelude::*;
use atom_camera::{setting::CameraSetting, CameraManagerPlugin, CameraTracker};
use atom_utils::{
    follow::{Following, RelativeTransform, RelativeTransformFreedom},
    transform::*,
};
use bevy::{
    core_pipeline::bloom::{BloomCompositeMode, BloomSettings},
    pbr::{ScreenSpaceAmbientOcclusionQualityLevel, ScreenSpaceAmbientOcclusionSettings},
    prelude::*,
};
use bevy_atmosphere::plugin::AtmosphereCamera;
use leafwing_input_manager::{prelude::InputMap, InputManagerBundle};
use lightyear::{prelude::client::Predicted, shared::replication::components::Controlled};
use terrain::TerrainObserver;

use crate::input::setting::PlayerAction;

#[derive(Debug, Default)]
pub struct GameCameraPlugin;

impl Plugin for GameCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraManagerPlugin)
            .add_systems(Startup, init_camera)
            .observe(attach_camera);
    }
}

fn init_camera(
    mut commands: Commands,
    camera_setting: Res<CameraSetting>,
    mut camera_tracking: ResMut<CameraTracker>,
) {
    let camera_entity = commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_translation(Vec3::new(0.0, -10.0, -10.0))
                    // transform: Transform::from_translation(Vec3::new(-10.0, -10.0, -10.0))
                    .looking_at(Vec3::ZERO, Dir3::Y),
                ..Default::default()
            },
            ScreenSpaceAmbientOcclusionSettings {
                quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
            },
            BloomSettings {
                intensity: 0.0,
                composite_mode: BloomCompositeMode::Additive,
                ..Default::default()
            },
            AtmosphereCamera::default(),
            TerrainObserver,
            InputManagerBundle::with_map(camera_setting.camera_input_map.clone()),
        ))
        .id();

    camera_tracking.set_main_camera(camera_entity);
}

fn attach_camera(
    trigger: Trigger<OnAdd, InputMap<PlayerAction>>,
    mut commands: Commands,
    camera_tracking: Res<CameraTracker>,
    locally_query: Query<(), (With<Predicted>, With<Controlled>)>,
) {
    let player_entity = trigger.entity();

    let Ok(()) = locally_query.get(player_entity) else {
        warn!("player is not locally controlled, skip attach camera.");
        return;
    };

    let Some(camera_entity) = camera_tracking.get_main_camera() else {
        warn!("main camera not found, skip attach camera.");
        return;
    };

    commands
        .entity(camera_entity)
        .insert((
            RelativeTransform(
                Transform::from_translation(Vec3::new(0.0, 10.0, -10.0))
                    .looking_at(Vec3::ZERO, Dir3::Y),
            ),
            RelativeTransformFreedom(TransformFreedom::Lock(TransformLockedFreedom {
                locked_translation: Some(TranslationLockedFreedom {
                    locked_x: false,
                    locked_y: false,
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
        .set::<Following>(player_entity);

    info!("attach camera done.");
}
