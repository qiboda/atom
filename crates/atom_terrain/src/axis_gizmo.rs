//! 屏幕左下角世界坐标轴指示器（类似 Unity Scene Gizmo）。
//!
//! 使用独立相机 + viewport + 窄长方体轴线实体。
//! 轴线固定在屏幕左下角 128×128 像素区域内，旋转跟随主相机。

use bevy::{
    prelude::*,
    color::palettes::css,
};
use bevy_camera::{
    Viewport,
    visibility::RenderLayers,
};

/// Gizmo 相机标记
#[derive(Component)]
struct GizmoCamera;

const GIZMO_LAYER: RenderLayers = RenderLayers::layer(1);

/// 屏幕空间坐标轴指示器插件。
pub struct AxisGizmoPlugin;

impl Plugin for AxisGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_gizmo);
        app.add_systems(PostStartup, restrict_main_camera);
        app.add_systems(Update, sync_gizmo);
    }
}

/// 确保主相机只渲染 layer 0。
fn restrict_main_camera(
    mut cameras: Query<&mut RenderLayers, (With<Camera3d>, Without<GizmoCamera>)>,
) {
    for mut layers in cameras.iter_mut() {
        // RenderLayers::default() has all layers; if it's default, restrict to layer 0
        if *layers == RenderLayers::default() {
            *layers = RenderLayers::layer(0);
        }
    }
}

/// 创建轴线 mesh 实体 + gizmo 相机。
fn setup_gizmo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let len = 0.8;
    let t = 0.05; // thickness

    // X 轴（红）
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(len, t, t))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: css::RED.into(),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(len * 0.5, 0.0, 0.0),
        GIZMO_LAYER,
    ));
    // Y 轴（绿）
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(t, len, t))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: css::GREEN.into(),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, len * 0.5, 0.0),
        GIZMO_LAYER,
    ));
    // Z 轴（蓝）
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(t, t, len))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: css::BLUE.into(),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.0, len * 0.5),
        GIZMO_LAYER,
    ));
    // 原点小球
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(t * 1.5))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: css::WHITE.into(),
            unlit: true,
            ..default()
        })),
        Transform::default(),
        GIZMO_LAYER,
    ));

    // Gizmo 相机（左下角，viewport 由 sync_gizmo 动态设置）
    commands.spawn((
        Camera3d::default(),
        Camera {
            order: 1,
            viewport: Some(Viewport {
                physical_position: UVec2::new(4, 4),
                physical_size: UVec2::new(256, 256),
                depth: 0.0..1.0,
            }),
            clear_color: ClearColorConfig::None,
            ..default()
        },
        Transform::from_xyz(2.2, 0.35, 2.2).looking_at(Vec3::ZERO, Vec3::Y),
        GIZMO_LAYER,
        GizmoCamera,
    ));
}

/// 同步 gizmo 相机旋转到主相机。
/// 同步 gizmo 相机旋转和 viewport 位置到窗口左下角。
fn sync_gizmo(
    main_cam: Query<&Transform, (With<Camera3d>, Without<GizmoCamera>)>,
    mut gizmo_cam: Query<
        (&mut Transform, &mut Camera),
        With<GizmoCamera>,
    >,
    windows: Query<&Window>,
) {
    let Ok(main) = main_cam.single() else { return };
    let Ok((mut gizmo, mut camera)) = gizmo_cam.single_mut() else { return };
    let Ok(window) = windows.single() else { return };

    // 跟随主相机旋转
    let dir = main.rotation * Vec3::new(-1.0, -0.25, -1.0).normalize();
    gizmo.translation = dir * 2.2;
    gizmo.look_at(Vec3::ZERO, Vec3::Y);

    // 固定到屏幕左下角
    let size: u32 = 256;
    let margin: u32 = 4;
    let physical_height = window.physical_size().y;
    let y = physical_height.saturating_sub(size + margin);
    camera.viewport = Some(Viewport {
        physical_position: UVec2::new(margin, y),
        physical_size: UVec2::splat(size),
        depth: 0.0..1.0,
    });
}
