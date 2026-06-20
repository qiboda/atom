//! 地形 MVP 示例 — observer 驱动多 chunk 动态加载。
//! 摄像机挂载 TerrainObserver，移动时自动加载/卸载地形 chunk。
//! `TerrainDebugConfig.wireframe` 控制线框显示。
//! Bevy 内置 FreeCamera: 鼠标旋转，WASD/QE 移动，右键按住锁定光标。
use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    prelude::*,
};
use atom_terrain::{
    debug::TerrainDebugConfig,
    loader::{TerrainObserver, TerrainObserverConfig},
    TerrainPlugin,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(TerrainPlugin::default());
    app.add_plugins(FreeCameraPlugin);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(mut commands: Commands) {
    // 开启线框调试
    commands.insert_resource(TerrainDebugConfig { wireframe: true });

    // 摄像机 + observer + FreeCamera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, -18.0, 15.0).looking_at(Vec3::new(0.0, -24.0, 0.0), Vec3::Y),
        FreeCamera {
            walk_speed: 5.0,
            run_speed: 15.0,
            ..default()
        },
        TerrainObserver,
        TerrainObserverConfig {
            load_radius: 3,
            height_range: -1..=1,
            margin: 1,
        },
    ));

    // 方向光
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
