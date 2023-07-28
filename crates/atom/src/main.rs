pub mod camera;
pub mod material;
pub mod renderdoc;
pub mod terrain;
pub mod ui;
pub mod visible;

use crate::renderdoc::RenderDocPlugin;
use bevy::{
    app::AppExit,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        settings::{WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_obj::ObjPlugin;
use bevy_xpbd_3d::prelude::PhysicsPlugins;
use camera::CameraControllerPlugin;
use material::CoolMaterial;
use terrain::{settings::TerrainSettings, TerrainPlugin};
use ui::FrameUIPlugin;
use visible::visible::VisibleTerrainRange;

fn main() {
    let mut app = App::new();

    app.insert_resource(TerrainSettings::new(1.0, 16))
        .add_plugins(RenderDocPlugin)
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                wgpu_settings: WgpuSettings {
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                },
            }),
            ObjPlugin,
            // WireframePlugin,
            // PhysicsPlugins::default(),
        ))
        .add_plugins(CameraControllerPlugin::default())
        .add_plugins(TerrainPlugin::default())
        .add_plugins(FrameUIPlugin)
        .add_plugins(MaterialPlugin::<CoolMaterial>::default())
        .add_systems(Startup, startup)
        .add_systems(Last, exit_game)
        .run();

    // app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());
    // bevy_mod_debugdump::print_main_schedule(&mut app);
    // bevy_mod_debugdump::print_render_graph(&mut app);
}

// #[bevycheck::system]
fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut cool_materials: ResMut<Assets<CoolMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    // mut wireframe_config: ResMut<WireframeConfig>,
) {
    // wireframe_config.global = true;

    commands.insert_resource(ClearColor(Color::rgb(1.0, 0.2, 0.1)));
    commands.insert_resource(Msaa::Sample4);
    commands.insert_resource(AmbientLight {
        color: Color::Rgba {
            red: 0.3,
            green: 0.3,
            blue: 0.3,
            alpha: 1.0,
        },
        brightness: 1.0,
    });

    // let mut material: StandardMaterial = Color::rgb(0.0, 0.0, 0.0).into();
    // material.double_sided = true;
    // // material.cull_mode = None;
    //
    // commands.spawn(MaterialMeshBundle::<StandardMaterial> {
    //     mesh: asset_server.load("blend.obj"),
    //     material: materials.add(material),
    //     transform: Transform::from_xyz(0.0, 0.0, 0.0),
    //     ..default()
    // });
    //
    //
    // commands.spawn(MaterialMeshBundle::<CoolMaterial> {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     material: cool_materials.add(CoolMaterial {
    //         color: Color::rgb(0.0, 1.0, 0.0),
    //         normal: Vec3::new(1.0, 0.0, 0.0),
    //         color_texture: asset_server.load("screenshot_jiumeizi.png"),
    //     }),
    //     transform: Transform::from_xyz(3.0, 0.0, 0.0),
    //     ..default()
    // });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(100.0, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    let size = 1.0 * 32.0;

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
        BloomSettings {
            intensity: 0.0,
            composite_mode: BloomCompositeMode::Additive,
            ..Default::default()
        },
        VisibleTerrainRange {
            min: Vec3::new(-size, -size, -size),
            max: Vec3::new(size, size, size),
        },
    ));
}

fn exit_game(keyboard_input: Res<Input<KeyCode>>, mut app_exit_events: EventWriter<AppExit>) {
    if keyboard_input.just_released(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
}
