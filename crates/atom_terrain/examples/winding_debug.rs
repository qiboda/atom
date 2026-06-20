//! Phase 2 vs Phase 3 对比测试。
//! 参数 `--phase2` 使用 TerrainPlugin (per-chunk)，
//! 默认使用 GlobalTerrainPlugin (global edge graph)。
use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    prelude::*,
};
use atom_terrain::{
    debug::TerrainDebugConfig,
    GlobalTerrainPlugin,
    TerrainPlugin,
};

fn main() {
    let use_phase2 = std::env::args().any(|a| a == "--phase2");

    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    if use_phase2 {
        info!("Using Phase 2 (TerrainPlugin)");
        app.add_plugins(TerrainPlugin::default());
    } else {
        info!("Using Phase 3 (GlobalTerrainPlugin)");
        app.add_plugins(GlobalTerrainPlugin::default());
    }
    app.add_plugins(FreeCameraPlugin);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(mut commands: Commands) {
    let use_phase2 = std::env::args().any(|a| a == "--phase2");

    commands.insert_resource(TerrainDebugConfig {
        wireframe: true,
        double_sided: true,
    });

    let mut cam = commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, -18.0, 15.0).looking_at(Vec3::new(0.0, -24.0, 0.0), Vec3::Y),
        FreeCamera {
            walk_speed: 5.0,
            run_speed: 15.0,
            ..default()
        },
    ));

    if use_phase2 {
        // Phase 2: 需要 loader 的 TerrainObserver 组件来触发 chunk 加载
        cam.insert((
            atom_terrain::loader::TerrainObserver,
            atom_terrain::loader::TerrainObserverConfig {
                load_radius: 3,
                height_range: -1..=1,
                margin: 1,
            },
        ));
    }

    commands.spawn((
        DirectionalLight {
            illuminance: 4000.0,
            ..default()
        },
        Transform::from_xyz(0.0, 20.0, 0.0).looking_at(Vec3::new(0.0, -24.0, 0.0), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 2000.0,
            ..default()
        },
        Transform::from_xyz(20.0, -24.0, 20.0).looking_at(Vec3::new(0.0, -24.0, 0.0), Vec3::Y),
    ));
}
