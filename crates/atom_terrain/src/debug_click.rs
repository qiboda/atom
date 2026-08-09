//! 鼠标点击 SDF 采样调试。
//!
//! 左键点击 → 从相机发射射线 → 沿射线 SDF marching 找到表面交点 →
//! 输出世界坐标、chunk ID、grid 坐标、是否在 chunk 边界。

use crate::compute::chunk::ChunkManager;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

/// 高度场 SDF（与 shader sdf_fill.wgsl 一致）
fn sdf_at(pos: Vec3) -> f32 {
    let h = height_at(pos.x, pos.z);
    pos.y - h
}

fn height_at(x: f32, z: f32) -> f32 {
    (x * 0.08).sin() * (z * 0.08).cos() * 8.0 - 24.0
}

/// 鼠标点击调试系统
pub fn debug_click_system(
    btn: Res<ButtonInput<MouseButton>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    manager: Res<ChunkManager>,
) {
    if !btn.just_pressed(MouseButton::Left) {
        return;
    }

    let Some(window) = q_window.iter().next() else {
        return;
    };
    let Some(cursor) = window.cursor_position() else {
        return;
    };

    let Some((camera, cam_transform)) = q_camera.iter().next() else {
        return;
    };

    // 从屏幕光标构建世界射线
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor) else {
        return;
    };

    let origin = ray.origin;
    let dir = ray.direction.normalize();

    // SDF ray marching — 简单线性搜索
    let step = 0.5; // 米
    let max_dist = 500.0;
    let mut t = 0.0;
    let mut last_sdf = sdf_at(origin);
    let mut hit = None;

    while t < max_dist {
        let pos = origin + dir * t;
        let d = sdf_at(pos);
        if last_sdf.signum() != d.signum() && last_sdf.abs() > 0.001 {
            // 跨过表面，取中点
            let prev = origin + dir * (t - step);
            hit = Some((prev + pos) * 0.5);
            break;
        }
        last_sdf = d;
        t += step;
    }

    let Some(hit_pos) = hit else {
        bevy::log::info!("[click] no surface hit in {:.0}m", max_dist);
        return;
    };

    let cid = manager.world_to_chunk(hit_pos);
    let min = cid.world_min();
    let local = hit_pos - min;
    let voxel_size = manager.voxel_size;
    let gx = (local.x / voxel_size) as u32;
    let gy = (local.y / voxel_size) as u32;
    let gz = (local.z / voxel_size) as u32;
    let gs = 32u32;

    // 检测是否在 chunk 边界附近
    let near_boundary = gx <= 1 || gx >= gs - 1 || gz <= 1 || gz >= gs - 1;
    let boundary_info = if near_boundary {
        format!(" BOUNDARY [gx={} gz={} gs={}]", gx, gz, gs)
    } else {
        String::new()
    };

    // 采样该 cell 的 8 个角密度值
    let vx = gx;
    let vy = gy;
    let vz = gz;
    let mut corners = String::new();
    for zi in 0..=1u32 {
        for yi in 0..=1u32 {
            for xi in 0..=1u32 {
                let px = min.x + (vx + xi) as f32 * voxel_size;
                let py = min.y + (vy + yi) as f32 * voxel_size;
                let pz = min.z + (vz + zi) as f32 * voxel_size;
                let d = sdf_at(Vec3::new(px, py, pz));
                corners.push_str(&format!(" ({},{},{})={:.2}", vx + xi, vy + yi, vz + zi, d));
            }
        }
    }

    bevy::log::info!(
        "[click] world=({:.1},{:.1},{:.1}) chunk=({},{},{}) grid=({},{},{}) sdf={:.3}{}{}",
        hit_pos.x,
        hit_pos.y,
        hit_pos.z,
        cid.x,
        cid.y,
        cid.z,
        vx,
        vy,
        vz,
        sdf_at(hit_pos),
        boundary_info,
        corners,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::chunk::ChunkManager;
    use bevy::camera::{ComputedCameraValues, ImageRenderTarget, RenderTargetInfo};
    use bevy::image::Image;
    use bevy::input::ButtonInput;
    use bevy::window::{PrimaryWindow, Window};
    use bevy::{MinimalPlugins, camera::RenderTarget};

    fn assert_approx(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-4,
            "expected {a} ≈ {b} but diff = {}",
            (a - b).abs()
        );
    }

    fn click_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<ButtonInput<MouseButton>>();
        app.insert_resource(ChunkManager::new(50.0, -50.0, 10.0, 42));
        app.add_systems(Update, debug_click_system);
        app
    }

    fn press_left(app: &mut App) {
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
    }

    fn spawn_window_with_cursor(app: &mut App) {
        let mut window = Window::default();
        window.set_cursor_position(Some(Vec2::new(400.0, 300.0)));
        app.world_mut().spawn((window, PrimaryWindow));
    }

    fn camera_with_viewport(app: &mut App, gt: GlobalTransform) {
        let camera = Camera {
            computed: ComputedCameraValues {
                target_info: Some(RenderTargetInfo {
                    physical_size: UVec2::new(800, 600),
                    scale_factor: 1.0,
                }),
                ..default()
            },
            ..default()
        };
        app.world_mut().spawn((camera, gt));
    }

    // ── 早期返回分支 ──

    #[test]
    fn returns_when_no_click() {
        let mut app = click_app();
        app.update();
    }

    #[test]
    fn returns_without_window() {
        let mut app = click_app();
        press_left(&mut app);
        app.update();
    }

    #[test]
    fn returns_without_cursor() {
        let mut app = click_app();
        app.world_mut().spawn((Window::default(), PrimaryWindow));
        press_left(&mut app);
        app.update();
    }

    #[test]
    fn returns_without_camera() {
        let mut app = click_app();
        spawn_window_with_cursor(&mut app);
        press_left(&mut app);
        app.update();
    }

    #[test]
    fn returns_when_viewport_to_world_fails() {
        let mut app = click_app();
        spawn_window_with_cursor(&mut app);
        // camera render target 指向无效 image handle → 无 target size → 射线构建失败
        app.world_mut().spawn((
            Camera::default(),
            RenderTarget::Image(ImageRenderTarget {
                handle: Handle::<Image>::default(),
                scale_factor: 1.0,
            }),
            GlobalTransform::IDENTITY,
        ));
        press_left(&mut app);
        app.update();
    }

    // ── 射线行进 ──

    #[test]
    fn ray_march_without_hit_logs_and_returns() {
        let mut app = click_app();
        spawn_window_with_cursor(&mut app);
        // 默认朝向（identity，y=0 视点沿 -Z）：地形最高 -16 → 永不穿越表面
        camera_with_viewport(&mut app, GlobalTransform::IDENTITY);
        press_left(&mut app);
        app.update();
    }

    #[test]
    fn ray_march_hits_surface_and_logs_debug_info() {
        let mut app = click_app();
        spawn_window_with_cursor(&mut app);
        // 相机俯视 -Y，x=1 处地表高度非采样点整数 → 穿越可被 sign change 检测
        let gt: GlobalTransform = Transform::from_translation(Vec3::new(1.0, 0.0, 1.0))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .into();
        camera_with_viewport(&mut app, gt);
        press_left(&mut app);
        app.update();
    }

    // ── 纯函数 ──

    #[test]
    fn sdf_zero_on_surface() {
        assert_approx(sdf_at(Vec3::new(0.0, height_at(0.0, 0.0), 0.0)), 0.0);
        assert_approx(sdf_at(Vec3::new(2.5, height_at(2.5, -1.0), -1.0)), 0.0);
    }

    #[test]
    fn sdf_sign_above_and_below_surface() {
        let h = height_at(3.0, -4.0);
        assert!(sdf_at(Vec3::new(3.0, h + 1.0, -4.0)) > 0.0, "表面上方为正");
        assert!(sdf_at(Vec3::new(3.0, h - 1.0, -4.0)) < 0.0, "表面下方为负");
    }

    #[test]
    fn height_at_bounded_by_amplitude_and_offset() {
        // (x*0.08).sin()*(z*0.08).cos()*8 - 24 → [-32, -16]
        for &(x, z) in &[(0.0, 0.0), (1.0, 2.0), (-5.0, 3.0), (100.0, -50.0)] {
            let h = height_at(x, z);
            assert!(
                (-32.0..=-16.0).contains(&h),
                "height_at({x},{z}) = {h} 超出 [-32,-16]"
            );
        }
    }

    #[test]
    fn height_at_periodic_in_sin_cos_cycle() {
        // sin/cos 周期 2π/0.08；移动一个周期后值复原
        let period = std::f32::consts::TAU / 0.08;
        let a = height_at(10.0, 10.0);
        let b = height_at(10.0 + period, 10.0 + period);
        assert_approx(a, b);
    }
}
