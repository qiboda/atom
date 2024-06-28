pub mod camera;
pub mod terrain;
pub mod ui;
pub mod visible;
pub mod window;

use bevy::asset::AssetMetaCheck;
use bevy::core::TaskPoolThreadAssignmentPolicy;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::pbr::{ScreenSpaceAmbientOcclusionQualityLevel, ScreenSpaceAmbientOcclusionSettings};
use bevy::render::settings::RenderCreation;
use bevy::{
    app::AppExit,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    log::LogPlugin,
    prelude::*,
    render::{
        settings::{WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
// use bevy_obj::ObjPlugin;
// use bevy_xpbd_3d::plugins::PhysicsPlugins;
use log_layers::{file_layer, LogLayersPlugin};
use settings::SettingPlugin;
use terrain::settings::{TerrainChunkSettings, TerrainClipMapSettings};
use terrain_player_client::trace::terrain_layer;

use crate::window::toggle_vsync;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
use camera::CameraControllerPlugin;
use terrain::{settings::TerrainSettings, TerrainPlugin};
use ui::FrameUIPlugin;
use visible::visible_range::VisibleTerrainRange;

pub fn bevy_entry() -> App {
    let mut app = App::new();

    app.insert_resource(TerrainSettings {
        chunk_settings: TerrainChunkSettings { chunk_size: 16.0 },
        clipmap_settings: TerrainClipMapSettings::default(),
    })
    .add_plugins(
        LogLayersPlugin::default()
            .add_layer(terrain_layer)
            .add_layer(file_layer::file_layer),
    )
    .add_plugins((
        DefaultPlugins
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                synchronous_pipeline_compilation: false,
            })
            .set(AssetPlugin {
                file_path: "assets".to_string(),
                processed_file_path: "".to_string(),
                watch_for_changes_override: Some(false),
                mode: AssetMode::Unprocessed,
                meta_check: AssetMetaCheck::Never,
            })
            .set(TaskPoolPlugin {
                task_pool_options: TaskPoolOptions {
                    async_compute: TaskPoolThreadAssignmentPolicy {
                        min_threads: 1,
                        max_threads: usize::MAX,
                        percent: 0.25,
                    },
                    ..default()
                },
            })
            .set(LogPlugin {
                custom_layer: LogLayersPlugin::get_layer,
                ..default()
            }),
        // ObjPlugin,
        WireframePlugin,
        // WorldInspectorPlugin::new(),
        SettingPlugin::<TerrainSettings> {
            paths: Default::default(),
        },
        // PhysicsPlugins::default(),
    ))
    .add_plugins(CameraControllerPlugin)
    .add_plugins(TerrainPlugin)
    .add_plugins(FrameUIPlugin)
    .add_systems(Startup, startup)
    .add_systems(Last, (exit_game, toggle_vsync));

    app
}

// #[bevycheck::system]
fn startup(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut cool_materials: ResMut<Assets<CoolMaterial>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
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
            // projection: Projection::Orthographic(OrthographicProjection {
            //     near: 0.0,
            //     far: 100.0,
            //     viewport_origin: (0.5, 0.5).into(),
            //     scaling_mode: ScalingMode::WindowSize(10.0),
            //     scale: 0.1,
            //     ..Default::default()
            // }),
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

fn exit_game(keyboard_input: Res<ButtonInput<KeyCode>>, mut app_exit_events: EventWriter<AppExit>) {
    if keyboard_input.just_released(KeyCode::Escape) {
        app_exit_events.send(AppExit::Success);
    }
}
