pub mod camera;
pub mod material;
pub mod renderdoc;
pub mod terrain;
pub mod ui;

use crate::renderdoc::RenderDocPlugin;
use bevy::{
    app::AppExit,
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    prelude::*,
};
use camera::CameraControllerPlugin;
use material::CoolMaterial;
use terrain::{chunk::visible::VisibleTerrainRange, TerrainPlugin};
use ui::FrameUIPlugin;

fn main() {
    let mut app = App::new();

    app.add_plugin(RenderDocPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(CameraControllerPlugin::default())
        .add_plugin(TerrainPlugin::default())
        .add_plugin(FrameUIPlugin)
        .add_plugin(MaterialPlugin::<CoolMaterial>::default())
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut cool_materials: ResMut<Assets<CoolMaterial>>,
) {
    commands.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.1)));
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
        transform: Transform::from_xyz(100.0, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });

    let size = 4.0 * 16.0;

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(30.0, 30.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
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
