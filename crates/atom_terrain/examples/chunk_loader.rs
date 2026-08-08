//! 全局 DC 地形示例 — observer-centric，无 chunk 边界。
//! 摄像机移动时自动重建全局 mesh。
//! `TerrainDebugConfig.wireframe` 控制线框显示。
//! Bevy 内置 FreeCamera: 鼠标旋转，WASD/QE 移动，右键按住锁定光标。
use atom_terrain::{GlobalTerrainPlugin, debug::TerrainDebugConfig, debug_map};
use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    prelude::*,
};
use bevy_camera::visibility::RenderLayers;

fn main() {
    let mut app = App::new();
    // RenderDoc 必须在 DefaultPlugins 之前
    app.add_plugins(atom_renderdoc::RenderDocPlugin);
    app.add_plugins(DefaultPlugins);
    app.add_plugins(GlobalTerrainPlugin);
    app.add_plugins(FreeCameraPlugin);
    app.add_systems(Startup, setup);
    app.add_systems(Startup, debug_map::generate_debug_maps_system);
    app.run();
}

fn setup(mut commands: Commands) {
    commands.insert_resource(TerrainDebugConfig {
        wireframe: false,
        double_sided: true,
        show_chunk_bounds: true,
        show_world_axes: true,
    });
    // 摄像机 + FreeCamera（TerrainObserver 资源由 GlobalTerrainPlugin 自动更新）
    commands.spawn((
        Camera3d::default(),
        RenderLayers::layer(0),
        Transform::from_xyz(15.0, 15.0, 15.0).looking_at(Vec3::new(0.0, 5.0, 0.0), Vec3::Y),
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
        Transform::from_xyz(0.0, 30.0, 0.0).looking_at(Vec3::new(0.0, 5.0, 0.0), Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 2000.0,
            ..default()
        },
        Transform::from_xyz(20.0, 30.0, 20.0).looking_at(Vec3::new(0.0, 5.0, 0.0), Vec3::Y),
    ));
}
