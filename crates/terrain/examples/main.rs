use bevy::{
    color::palettes::css,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    log::{Level, LogPlugin},
    pbr::{
        wireframe::{WireframeConfig, WireframePlugin},
        ScreenSpaceAmbientOcclusionQualityLevel, ScreenSpaceAmbientOcclusionSettings,
    },
    prelude::*,
};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin, PlayerPlugin};
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
            filter: "wgpu=error,naga=warn,terrain=info".to_string(),
            ..default()
        }),
        // ObjPlugin,
        WireframePlugin,
    ))
    .add_plugins(TerrainSubsystemPlugin)
    .add_plugins(NoCameraPlayerPlugin)
    .add_systems(Startup, startup)
    .run();
}

fn startup(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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

    // commands.spawn(MaterialMeshBundle {
    //     mesh: meshes.add(Mesh::from(Cuboid {
    //         half_size: Vec3::splat(2.0),
    //     })),
    //     material: materials.add(StandardMaterial {
    //         base_color: LinearRgba::WHITE.into(),
    //         ..Default::default()
    //     }),
    //     transform: Transform::from_xyz(8.0, 8.0, 8.0),
    //     ..Default::default()
    // });

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
        FlyCam,
        VisibleTerrainRange::new(Vec3::splat(size)),
    ));
}
