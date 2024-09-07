use atom_internal::plugins::AtomDefaultPlugins;
use atom_renderdoc::RenderDocPlugin;
use avian3d::PhysicsPlugins;
use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    log::LogPlugin,
    pbr::{
        wireframe::WireframePlugin, ScreenSpaceAmbientOcclusionQualityLevel,
        ScreenSpaceAmbientOcclusionSettings,
    },
    prelude::*,
    render::{camera::RenderTarget, diagnostic::RenderDiagnosticsPlugin},
    window::WindowRef,
};
use bevy_flycam::{FlyCam, MovementSettings, NoCameraPlayerPlugin};
use bevy_mod_picking::{
    events::{Click, Move, Pointer},
    prelude::RaycastBackend,
    DefaultPickingPlugins,
};
use bevy_water::{WaterPlugin, WaterSettings};
use dotenv::dotenv;
use log_layers::LogLayersPlugin;
// use oxidized_navigation::{
//     debug_draw::{DrawNavMesh, OxidizedNavigationDebugDrawPlugin},
//     NavMeshSettings, OxidizedNavigationPlugin,
// };
use terrain::{
    isosurface::csg::event::{CSGOperateApplyEvent, CSGOperateType, CSGPrimitive},
    lod::lod_gizmos::TerrainLodGizmosPlugin,
    map::compute_height::TerrainMapTextures,
    TerrainObserver, TerrainSubsystemPlugin,
};

pub fn main() {
    dotenv().ok();

    let mut app = App::new();

    app.add_plugins(
        AtomDefaultPlugins
            .set(LogPlugin {
                custom_layer: LogLayersPlugin::get_layer,
                filter: "wgpu=error,naga=warn,terrain=info".to_string(),
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
    // 固定帧率
    // .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(0.5)))
    // .add_plugins((
    //     OxidizedNavigationPlugin::<Collider>::new(NavMeshSettings::from_agent_and_bounds(
    //         1.0, 2.0, 300.0, -100.0,
    //     )),
    //     OxidizedNavigationDebugDrawPlugin,
    // ))
    .add_plugins(
        // We want to disable the raycast backend because it is enabled by default. All supplied
        // backends are optional. In your app, you can disable the default features of the
        // plugin and only enable the backends you want to use. Picking will still work if both
        // backends are enabled, but that would mean we wouldn't be able to test the Avian
        // backend in isolation.
        DefaultPickingPlugins.build().disable::<RaycastBackend>(),
    )
    .add_plugins((
        PhysicsPlugins::default(),
        // PhysicsDebugPlugin::default()
    ))
    // .insert_resource(DebugPickingMode::Normal)
    .add_plugins(RenderDocPlugin)
    .add_plugins(RenderDiagnosticsPlugin)
    .add_plugins(WireframePlugin)
    .add_plugins(TerrainSubsystemPlugin)
    .add_plugins(TerrainLodGizmosPlugin)
    .add_plugins(NoCameraPlayerPlugin)
    .insert_resource(WaterSettings {
        height: 0.0,
        ..default()
    })
    .add_plugins(WaterPlugin)
    .add_systems(Startup, startup)
    .add_systems(
        Update,
        (
            update_terrain_observer,
            update_sprite_texture,
            change_camera_speed,
            pointer_click_terrain,
        ),
    )
    // .add_plugins(WorldInspectorPlugin::new())
    .insert_resource(MovementSettings {
        speed: 30.0,
        ..default()
    })
    .run();
}

#[derive(Component)]
struct PlayerCamera;

fn startup(
    mut commands: Commands,
    // mut nav_mesh: ResMut<DrawNavMesh>,
    terrain_height_map_image: Option<ResMut<TerrainMapTextures>>,
) {
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
        brightness: 1000.0,
    });

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
        PlayerCamera,
        // TerrainObserver,
    ));

    if let Some(image) = terrain_height_map_image {
        let second_window = commands
            .spawn(Window {
                title: "Second window".to_owned(),
                present_mode: bevy::window::PresentMode::AutoNoVsync,
                ..default()
            })
            .id();

        let second_camera = commands
            .spawn(Camera2dBundle {
                camera: Camera {
                    target: RenderTarget::Window(WindowRef::Entity(second_window)),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                ..default()
            })
            .id();

        commands
            .spawn((NodeBundle::default(), TargetCamera(second_camera)))
            .with_children(|parent| {
                parent.spawn(SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(Vec2::new(1024.0, 1024.0)),
                        ..default()
                    },
                    texture: image.height_texture.clone(),
                    ..default()
                });
            });
    }
}

pub fn update_sprite_texture(
    terrain_height_map_image: ResMut<TerrainMapTextures>,
    mut sprite: Query<&mut Handle<Image>, With<Sprite>>,
) {
    if terrain_height_map_image.is_changed() {
        let mut sprite_texture = sprite.get_single_mut().unwrap();
        *sprite_texture = terrain_height_map_image.height_texture.clone();
    }
}

#[allow(clippy::type_complexity)]
fn update_terrain_observer(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    query: Query<(Entity, Option<&TerrainObserver>), (With<Camera>, With<PlayerCamera>)>,
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
        move_setting.speed = 1000.0;
    } else {
        move_setting.speed = 10.0;
    }
}

#[allow(dead_code)]
fn pointer_move_terrain(mut event_reader: EventReader<Pointer<Move>>, mut gizmos: Gizmos) {
    for event in event_reader.read() {
        if let Some(position) = event.event.hit.position {
            gizmos.cuboid(
                Transform::from_translation(position).with_scale(Vec3::splat(3.0)),
                LinearRgba::RED,
            );
        }
    }
}

fn pointer_click_terrain(
    mut event_reader: EventReader<Pointer<Click>>,
    mut event_writer: EventWriter<CSGOperateApplyEvent>,
) {
    for event in event_reader.read() {
        info!("pointer_click_terrain");
        if let Some(position) = event.event.hit.position {
            info!("pointer_click_terrain send event");
            event_writer.send(CSGOperateApplyEvent {
                transform: Transform::from_translation(position),
                primitive: CSGPrimitive::Box {
                    size: Vec3::splat(3.0),
                },
                operate_type: CSGOperateType::Difference,
            });
        }
    }
}
