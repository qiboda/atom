use atom_internal::plugins::AtomDefaultPlugins;
use bevy::{
    color::palettes::css,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    log::LogPlugin,
    math::{bounding::Aabb3d, Vec3A},
    pbr::{
        wireframe::{WireframeConfig, WireframePlugin},
        ScreenSpaceAmbientOcclusionQualityLevel, ScreenSpaceAmbientOcclusionSettings,
    },
    prelude::*,
};
use bevy_debug_grid::{Grid, GridAxis};
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use bevy_xpbd_3d::prelude::Collider;
use dotenv::dotenv;
use log_layers::LogLayersPlugin;
use oxidized_navigation::{
    debug_draw::{DrawNavMesh, OxidizedNavigationDebugDrawPlugin},
    NavMeshSettings, OxidizedNavigationPlugin,
};
use terrain::{
    chunk_mgr::chunk_loader::TerrainChunkLoader,
    isosurface::surface::{
        csg::{csg_shapes::CSGCube, CSGOperation},
        event::CSGOperationEndEvent,
        shape_surface::IsosurfaceContext,
    },
    TerrainObserver, TerrainSubsystemPlugin,
};
use vleue_navigator::prelude::NavMeshBundle;

pub fn main() {
    dotenv().ok();

    let mut app = App::new();

    app.add_plugins(
        AtomDefaultPlugins
            .set(LogPlugin {
                custom_layer: LogLayersPlugin::get_layer,
                filter: "info,wgpu=error,naga=warn,terrain=info".to_string(),
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // uncomment for unthrottled FPS
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            }),
    )
    // .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(0.5)))
    .add_plugins((
        OxidizedNavigationPlugin::<Collider>::new(NavMeshSettings::from_agent_and_bounds(
            1.0, 2.0, 300.0, -100.0,
        )),
        OxidizedNavigationDebugDrawPlugin,
    ))
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
    .add_systems(
        Update,
        (
            update_terrain_observer,
            change_camera_speed,
            apply_csg_operation,
        ),
    )
    // .add_plugins(WorldInspectorPlugin::new())
    .insert_resource(MovementSettings {
        speed: 300.0,
        ..default()
    })
    .run();
}

fn startup(
    mut commands: Commands,
    mut wireframe_config: ResMut<WireframeConfig>,
    mut nav_mesh: ResMut<DrawNavMesh>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    nav_mesh.0 = true;
    wireframe_config.global = true;

    // fn startup(mut commands: Commands) {
    commands.spawn((
        Grid {
            // Space between each line
            spacing: 64.0,
            // Line count along a single axis
            count: 256,
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

    commands.spawn(MaterialMeshBundle {
        mesh: meshes.add(Mesh::from(Plane3d {
            normal: Dir3::Y,
            half_size: Vec2::splat(65536.0 * 0.5),
        })),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            unlit: true,
            ..default()
        }),
        ..default()
    });

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
        // TerrainObserver,
    ));
}

pub fn update_terrain_observer(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    query: Query<(Entity, Option<&TerrainObserver>), With<Camera>>,
) {
    if input.just_pressed(KeyCode::KeyK) {
        for (entity, observer) in query.iter() {
            if observer.is_none() {
                commands.entity(entity).insert(TerrainObserver);
            } else {
                commands.entity(entity).remove::<TerrainObserver>();
            }
        }
    }
}

pub fn change_camera_speed(
    input: Res<ButtonInput<KeyCode>>,
    mut move_setting: ResMut<MovementSettings>,
) {
    if input.pressed(KeyCode::ControlLeft) {
        move_setting.speed = 3000.0;
    } else {
        move_setting.speed = 3.0;
    }
}

pub fn apply_csg_operation(
    input: Res<ButtonInput<KeyCode>>,
    context: ResMut<IsosurfaceContext>,
    mut commands: Commands,
    mut loader: ResMut<TerrainChunkLoader>,
) {
    if input.just_pressed(KeyCode::KeyJ) {
        let mut shape_surface = context.shape_surface.write().unwrap();
        shape_surface.apply_csg_operation(
            Box::new(CSGCube {
                location: Vec3::new(5.0, 0.0, 5.0),
                half_size: Vec3::splat(3.0),
            }),
            CSGOperation::Difference,
        );
        commands.trigger(CSGOperationEndEvent {
            aabb: Aabb3d {
                min: Vec3A::new(5.0, 0.0, 5.0) - Vec3A::splat(3.0),
                max: Vec3A::new(5.0, 0.0, 5.0) + Vec3A::splat(3.0),
            },
        });
        loader.add_reload_aabb(Aabb3d {
            min: Vec3A::new(5.0, 0.0, 5.0) - Vec3A::splat(3.0),
            max: Vec3A::new(5.0, 0.0, 5.0) + Vec3A::splat(3.0),
        });
    }
}
