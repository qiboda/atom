use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    log::LogPlugin,
    pbr::{
        wireframe::{WireframeConfig, WireframePlugin},
        ScreenSpaceAmbientOcclusionQualityLevel, ScreenSpaceAmbientOcclusionSettings,
    },
    prelude::*,
};
use log_layers::{file_layer, LogLayersPlugin};
use settings::{SettingSourceConfig, SettingsPlugin};
use terrain::{visible::visible_range::VisibleTerrainRange, TerrainSubsystemPlugin};

pub fn main() {
    let mut app = App::new();

    app.add_plugins(SettingsPlugin {
        game_source_config: SettingSourceConfig {
            source_id: "config_terrain".into(),
            base_path: "config/terrain".into(),
        },
        user_source_config: SettingSourceConfig {
            source_id: "config_terrain".into(),
            base_path: "config/terrain".into(),
        },
    })
    .add_plugins((
        LogLayersPlugin::default().add_layer(file_layer::file_layer),
        DefaultPlugins.set(LogPlugin {
            custom_layer: LogLayersPlugin::get_layer,
            ..default()
        }),
        // ObjPlugin,
        WireframePlugin,
    ))
    .add_plugins(TerrainSubsystemPlugin)
    .add_plugins(CameraControllerPlugin)
    .add_systems(Startup, startup)
    .run();
}

fn startup(mut commands: Commands, mut wireframe_config: ResMut<WireframeConfig>) {
    wireframe_config.global = true;

    commands.insert_resource(ClearColor(LinearRgba::new(0.3, 0.2, 0.1, 1.0).into()));
    commands.insert_resource(Msaa::Sample4);
    commands.insert_resource(AmbientLight {
        color: LinearRgba {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            alpha: 1.0,
        }
        .into(),
        brightness: 0.3,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            color: Color::WHITE,
            illuminance: 100000.0,
            shadows_enabled: true,
            shadow_depth_bias: 0.0,
            shadow_normal_bias: 0.0,
        },
        transform: Transform::from_xyz(100.0, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    let size = 4.0 * 16.0;

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(8.0, 8.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                hdr: true,
                order: 0,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        ScreenSpaceAmbientOcclusionSettings {
            quality_level: ScreenSpaceAmbientOcclusionQualityLevel::High,
        },
        BloomSettings {
            intensity: 0.0,
            composite_mode: BloomCompositeMode::Additive,
            ..Default::default()
        },
        VisibleTerrainRange::new(Vec3::splat(size)),
    ));
}

#[derive(Default)]
pub struct CameraControllerPlugin;

impl Plugin for CameraControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_movement);
    }
}

fn camera_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut query: Query<(&GlobalTransform, &mut Transform), With<Camera3d>>,
) {
    const MOVE_SPEED: f32 = 10.0;
    const ROTATION_SPEED: f32 = 0.5;
    for (global_transform, mut trans) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::KeyW) {
            trans.translation += MOVE_SPEED * global_transform.forward() * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::KeyS) {
            trans.translation += MOVE_SPEED * global_transform.back() * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::KeyA) {
            trans.translation += MOVE_SPEED * global_transform.left() * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::KeyD) {
            trans.translation += MOVE_SPEED * global_transform.right() * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            trans.translation += MOVE_SPEED * Vec3::new(0.0, -1.0, 0.0) * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::ShiftRight) {
            trans.translation += MOVE_SPEED * Vec3::new(0.0, 1.0, 0.0) * time.delta_seconds();
        }

        if keyboard_input.pressed(KeyCode::KeyQ) {
            trans.rotate_y(-10.0 * time.delta_seconds() * ROTATION_SPEED);
        }

        if keyboard_input.pressed(KeyCode::KeyE) {
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
