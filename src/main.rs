pub mod camera;
pub mod debug;
pub mod material;
pub mod renderdoc;

use crate::renderdoc::RenderDocPlugin;
use bevy::{
    app::AppExit,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        clear_color::ClearColorConfig,
        tonemapping::Tonemapping,
    },
    // log::LogPlugin,
    prelude::*,
    render::camera::Viewport,
    window::WindowResized,
};
use camera::CameraControllerPlugin;
use debug::AtomDebugPlugin;
use material::CoolMaterial;

fn main() {
    let mut app = App::new();

    app.add_plugin(RenderDocPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraControllerPlugin::default())
        .add_plugin(MaterialPlugin::<CoolMaterial>::default())
        .add_plugin(AtomDebugPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, exit_game)
        .add_systems(Update, set_camera_viewports)
        .run();

    // app.add_plugins(DefaultPlugins.build().disable::<LogPlugin>());
    // bevy_mod_debugdump::print_main_schedule(&mut app);
    // bevy_mod_debugdump::print_render_graph(&mut app);
}

// #[bevycheck::system]
fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut cool_materials: ResMut<Assets<CoolMaterial>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.1)));
    commands.insert_resource(Msaa::Off);
    commands.insert_resource(AmbientLight {
        color: Color::Rgba {
            red: 0.3,
            green: 0.3,
            blue: 0.3,
            alpha: 1.0,
        },
        brightness: 1.0,
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(StandardMaterial {
            emissive: Color::rgb_linear(10.0, 1.0, 1.0),
            ..default()
        }),
        ..default()
    });

    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     material: materials.add(StandardMaterial::default()),
    //     transform: Transform::from_xyz(1.5, 0.0, 0.0),
    //     ..default()
    // });

    commands.spawn(MaterialMeshBundle::<CoolMaterial> {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: cool_materials.add(CoolMaterial {
            color: Color::rgb(0.0, 1.0, 0.0),
            normal: Vec3::new(1.0, 0.0, 0.0),
            color_texture: asset_server.load("screenshot_jiumeizi.png"),
        }),
        transform: Transform::from_xyz(3.0, 0.0, 0.0),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(4.0, 4.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                hdr: true,
                order: 0,
                msaa_writeback: false,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings {
            intensity: 0.9,
            composite_mode: BloomCompositeMode::Additive,
            ..Default::default()
        },
        LeftCamera,
    ));

    // // Right Camera
    // commands.spawn((
    //     Camera3dBundle {
    //         transform: Transform::from_xyz(-4.0, 4.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    //         camera: Camera {
    //             hdr: true,
    //             // Renders the right camera after the left camera, which has a default priority of 0
    //             msaa_writeback: false,
    //             order: 1,
    //             ..default()
    //         },
    //         // tonemapping: Tonemapping::TonyMcMapface,
    //         camera_3d: Camera3d {
    //             // don't clear on the second camera because the first camera already cleared the window
    //             clear_color: ClearColorConfig::None,
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     // BloomSettings {
    //     //     intensity: 0.2,
    //     //     composite_mode: BloomCompositeMode::Additive,
    //     //     ..Default::default()
    //     // },
    //     RightCamera,
    // ));
    //
    // commands.spawn((
    //     Camera3dBundle {
    //         transform: Transform::from_xyz(-4.0, 4.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    //         camera: Camera {
    //             hdr: true,
    //             // Renders the right camera after the left camera, which has a default priority of 0
    //             msaa_writeback: false,
    //             order: 1,
    //             ..default()
    //         },
    //         // tonemapping: Tonemapping::TonyMcMapface,
    //         camera_3d: Camera3d {
    //             // don't clear on the second camera because the first camera already cleared the window
    //             clear_color: ClearColorConfig::None,
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     // BloomSettings {
    //     //     intensity: 0.2,
    //     //     composite_mode: BloomCompositeMode::Additive,
    //     //     ..Default::default()
    //     // },
    //     TopCamera,
    // ));
    //
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-4.0, 4.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            camera: Camera {
                hdr: true,
                // Renders the right camera after the left camera, which has a default priority of 0
                msaa_writeback: false,
                order: 1,
                ..default()
            },
            // tonemapping: Tonemapping::TonyMcMapface,
            camera_3d: Camera3d {
                // don't clear on the second camera because the first camera already cleared the window
                clear_color: ClearColorConfig::None,
                ..default()
            },
            ..default()
        },
        // BloomSettings {
        //     intensity: 0.2,
        //     composite_mode: BloomCompositeMode::Additive,
        //     ..Default::default()
        // },
        DownCamera,
    ));
}

fn exit_game(keyboard_input: Res<Input<KeyCode>>, mut app_exit_events: EventWriter<AppExit>) {
    if keyboard_input.just_released(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
}

#[derive(Component)]
struct LeftCamera;

#[derive(Component)]
struct RightCamera;

#[derive(Component)]
struct TopCamera;

#[derive(Component)]
struct DownCamera;

fn set_camera_viewports(
    windows: Query<&Window>,
    mut resize_events: EventReader<WindowResized>,
    mut left_camera: Query<
        &mut Camera,
        (
            With<LeftCamera>,
            Without<RightCamera>,
            Without<TopCamera>,
            Without<DownCamera>,
        ),
    >,
    mut right_camera: Query<
        &mut Camera,
        (
            With<RightCamera>,
            Without<LeftCamera>,
            Without<TopCamera>,
            Without<DownCamera>,
        ),
    >,
    mut top_camera: Query<
        &mut Camera,
        (
            With<TopCamera>,
            Without<LeftCamera>,
            Without<RightCamera>,
            Without<DownCamera>,
        ),
    >,
    mut down_camera: Query<&mut Camera, With<DownCamera>>,
) {
    // We need to dynamically resize the camera's viewports whenever the window size changes
    // so then each camera always takes up half the screen.
    // A resize_event is sent when the window is first created, allowing us to reuse this system for initial setup.
    for resize_event in resize_events.iter() {
        let window = windows.get(resize_event.window).unwrap();
        let mut left_camera = left_camera.single_mut();
        left_camera.viewport = Some(Viewport {
            physical_position: UVec2::new(0, window.resolution.physical_height() / 10),
            physical_size: UVec2::new(
                window.resolution.physical_width() / 2,
                window.resolution.physical_height() / 4,
            ),
            ..default()
        });
    }
}
