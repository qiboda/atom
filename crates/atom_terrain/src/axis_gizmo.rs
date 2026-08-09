//! 屏幕左下角世界坐标轴指示器（render-to-texture，透明背景）。
//!
//! Gizmo 相机渲染到独立 256×256 纹理（自带 depth buffer）。
//! 纹理初始透明 + clear 透明 → 轴实体渲染在上 → UI ImageNode alpha 混合显示。
//! 背景区域全透明 → 主场景透出；轴实体不透明 → 遮挡主场景。
//!
//! 设计依据：Bevy 0.19 同一 render target 深度冲突无可避免 → 独立纹理根除。

use bevy::render::extract_component::{ExtractComponent, ExtractComponentPlugin};
use bevy::{
    asset::RenderAssetUsages,
    color::palettes::css,
    image::Image,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};
use bevy_camera::{Camera3dDepthLoadOp, visibility::RenderLayers};

#[derive(Component, Clone, ExtractComponent)]
pub(crate) struct GizmoCamera;

/// 屏幕空间坐标轴指示器插件。
pub struct AxisGizmoPlugin;

impl Plugin for AxisGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<GizmoCamera>::default());
        app.add_systems(Startup, setup_gizmo);
        app.add_systems(Update, sync_gizmo_camera);
    }
}

fn setup_gizmo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let len = 0.8;
    let t = 0.06;
    let mut mat = |color: Srgba| {
        materials.add(StandardMaterial {
            base_color: color.into(),
            unlit: true,
            ..default()
        })
    };
    let layer1 = || RenderLayers::layer(1);

    // 轴实体 — world origin，仅 gizmo 相机可见
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(len, t, t))),
        MeshMaterial3d(mat(css::RED)),
        Transform::from_xyz(len * 0.5, 0.0, 0.0),
        layer1(),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(t, len, t))),
        MeshMaterial3d(mat(css::GREEN)),
        Transform::from_xyz(0.0, len * 0.5, 0.0),
        layer1(),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(t, t, len))),
        MeshMaterial3d(mat(css::BLUE)),
        Transform::from_xyz(0.0, 0.0, len * 0.5),
        layer1(),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(t * 1.5))),
        MeshMaterial3d(mat(css::WHITE)),
        Transform::default(),
        layer1(),
    ));

    // 渲染目标纹理 — 全透明初始
    let size = 256u32;
    let pixel: [u8; 4] = [0, 0, 0, 0];
    let mut gizmo_image = Image::new_fill(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &pixel,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    gizmo_image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;
    let gizmo_handle = images.add(gizmo_image);

    // Gizmo 相机 — 独立纹理，透明 clear
    commands.spawn((
        Camera3d {
            depth_load_op: Camera3dDepthLoadOp::Clear(0.0),
            ..default()
        },
        Camera {
            order: 0,
            clear_color: ClearColorConfig::Custom(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            ..default()
        },
        Transform::from_xyz(2.2, 0.35, 2.2).looking_at(Vec3::ZERO, Vec3::Y),
        bevy::camera::RenderTarget::Image(gizmo_handle.clone().into()),
        layer1(),
        GizmoCamera,
    ));

    // UI ImageNode — 不设置 BackgroundColor，利用纹理 alpha 混合
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(4.0),
            left: Val::Px(4.0),
            width: Val::Px(size as f32),
            height: Val::Px(size as f32),
            ..default()
        },
        GlobalZIndex(i32::MAX),
        ImageNode::new(gizmo_handle),
    ));
}

fn sync_gizmo_camera(
    main_cam: Query<&Transform, (With<Camera3d>, Without<GizmoCamera>)>,
    mut gizmo_cam: Query<&mut Transform, With<GizmoCamera>>,
) {
    let Ok(main) = main_cam.single() else { return };
    let Ok(mut gt) = gizmo_cam.single_mut() else {
        return;
    };

    let dir = main.rotation * Vec3::new(-1.0, -0.25, -1.0).normalize();
    gt.translation = dir * 2.2;
    gt.look_at(Vec3::ZERO, Vec3::Y);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::MinimalPlugins;
    use bevy_camera::RenderTarget;

    fn assert_approx(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-5,
            "expected {a} ≈ {b} but diff = {}",
            (a - b).abs()
        );
    }

    #[test]
    fn setup_gizmo_spawns_axes_camera_and_ui() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<StandardMaterial>>();
        app.init_resource::<Assets<Image>>();
        app.add_systems(Update, setup_gizmo);

        app.update();

        // 3 轴 cuboid + 1 球 = 4 个 mesh 实体
        let mut mesh_q = app.world_mut().query::<(&Mesh3d, &GlobalTransform)>();
        assert_eq!(mesh_q.iter(app.world()).count(), 4);

        // 1 个 gizmo 相机，render target 是 Image 纹理
        let mut cam_q = app.world_mut().query::<(&GizmoCamera, &RenderTarget)>();
        let cams: Vec<_> = cam_q.iter(app.world()).collect();
        assert_eq!(cams.len(), 1);
        assert!(matches!(cams[0].1, RenderTarget::Image(_)));

        // 1 个 UI ImageNode
        let mut ui_q = app.world_mut().query::<&ImageNode>();
        assert_eq!(ui_q.iter(app.world()).count(), 1);

        // 注册了 1 张 render target 纹理
        let images = app.world().resource::<Assets<Image>>();
        assert_eq!(images.len(), 1);
    }

    #[test]
    fn sync_early_returns_without_main_camera() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, sync_gizmo_camera);
        let gizmo = app
            .world_mut()
            .spawn((GizmoCamera, Transform::from_xyz(1.0, 2.0, 3.0)))
            .id();

        app.update();

        assert_eq!(
            app.world()
                .entity(gizmo)
                .get::<Transform>()
                .expect("有 Transform")
                .translation,
            Vec3::new(1.0, 2.0, 3.0)
        );
    }

    #[test]
    fn sync_early_returns_without_gizmo_camera() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, sync_gizmo_camera);
        app.world_mut()
            .spawn((Camera3d::default(), Transform::IDENTITY));

        app.update();
    }

    #[test]
    fn sync_places_gizmo_on_rotated_direction_from_main() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, sync_gizmo_camera);
        app.world_mut().spawn((
            Camera3d::default(),
            Transform::from_rotation(bevy::math::Quat::from_rotation_y(0.5)),
        ));
        let gizmo = app
            .world_mut()
            .spawn((GizmoCamera, Transform::from_xyz(5.0, 5.0, 5.0)))
            .id();

        app.update();

        let t = app
            .world()
            .entity(gizmo)
            .get::<Transform>()
            .expect("有 Transform");
        assert_approx(t.translation.length(), 2.2);
        let dir = bevy::math::Quat::from_rotation_y(0.5)
            * bevy::math::Vec3::new(-1.0, -0.25, -1.0).normalize();
        assert_approx(t.translation.x, dir.x * 2.2);
        assert_approx(t.translation.y, dir.y * 2.2);
        assert_approx(t.translation.z, dir.z * 2.2);
    }
}
