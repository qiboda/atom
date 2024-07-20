use atom_internal::plugins::AtomDefaultPlugins;
use bevy::{
    color::palettes::css,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    log::LogPlugin,
    pbr::{
        wireframe::{WireframeConfig, WireframePlugin},
        ScreenSpaceAmbientOcclusionQualityLevel, ScreenSpaceAmbientOcclusionSettings,
    },
    prelude::*,
};
use bevy_debug_grid::{Grid, GridAxis};
use bevy_flycam::{FlyCam, NoCameraPlayerPlugin};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use log_layers::LogLayersPlugin;
use terrain::{visible::visible_range::VisibleTerrainRange, TerrainSubsystemPlugin};

pub fn main() {
    let mut app = App::new();

    app.add_plugins(AtomDefaultPlugins.set(LogPlugin {
        custom_layer: LogLayersPlugin::get_layer,
        filter: "wgpu=error,naga=warn,terrain=warn".to_string(),
        ..default()
    }))
    // .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(0.5)))
    .add_plugins(WireframePlugin)
    .add_plugins(TerrainSubsystemPlugin)
    .add_plugins(NoCameraPlayerPlugin)
    .add_plugins(FpsOverlayPlugin {
        config: FpsOverlayConfig {
            text_config: TextStyle {
                // Here we define size of our overlay
                font_size: 50.0,
                // We can also change color of the overlay
                color: Color::srgb(0.0, 1.0, 0.0),
                // If we want, we can use a custom font
                font: default(),
            },
        },
    })
    .add_systems(Startup, startup)
    .add_plugins(WorldInspectorPlugin::new())
    .run();
}

fn startup(mut commands: Commands, mut wireframe_config: ResMut<WireframeConfig>) {
    wireframe_config.global = true;

    // fn startup(mut commands: Commands) {
    commands.spawn((
        Grid {
            // Space between each line
            spacing: 16.0,
            // Line count along a single axis
            count: 16,
            // Color of the lines
            color: css::ORANGE.into(),
            // Alpha mode for all components
            alpha_mode: AlphaMode::Opaque,
        },
        // SubGrid {
        //     // Line count between each line of the main grid
        //     count: 16,
        //     // Line color
        //     color: css::GRAY.into(),
        // },
        GridAxis {
            x: Some(css::RED.into()),
            y: Some(css::GREEN.into()),
            z: Some(css::BLUE.into()),
        },
        TransformBundle::default(),
        VisibilityBundle::default(),
    ));

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

    let size = 16.0 * 16.0;

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(8.0, -0.1, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
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
