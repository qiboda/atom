//! 全局 DC 地形示例 — observer-centric，无 chunk 边界。
//! 摄像机移动时自动重建全局 mesh。
//! `TerrainDebugConfig.wireframe` 控制线框显示。
//! Bevy 内置 FreeCamera: 鼠标旋转，WASD/QE 移动，右键按住锁定光标。
use atom_terrain::{GlobalTerrainPlugin, debug::TerrainDebugConfig};
use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    prelude::*,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(GlobalTerrainPlugin::default());
    app.add_plugins(FreeCameraPlugin);
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(mut commands: Commands) {
    // 调试开关
    commands.insert_resource(TerrainDebugConfig {
        wireframe: true,
        double_sided: true,
    });

    // 摄像机 + FreeCamera（TerrainObserver 资源由 GlobalTerrainPlugin 自动更新）
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, -18.0, 15.0).looking_at(Vec3::new(0.0, -24.0, 0.0), Vec3::Y),
        FreeCamera {
            walk_speed: 5.0,
            run_speed: 15.0,
            ..default()
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
