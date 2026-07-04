//! 鼠标点击 SDF 采样调试。
//!
//! 左键点击 → 从相机发射射线 → 沿射线 SDF marching 找到表面交点 →
//! 输出世界坐标、chunk ID、grid 坐标、是否在 chunk 边界。

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crate::compute::chunk::ChunkManager;

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
    if !btn.just_pressed(MouseButton::Left) { return; }

    let Some(window) = q_window.iter().next() else { return };
    let Some(cursor) = window.cursor_position() else { return };

    let Some((camera, cam_transform)) = q_camera.iter().next() else { return };

    // 从屏幕光标构建世界射线
    let Ok(ray) = camera.viewport_to_world(cam_transform, cursor) else { return };

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
    let vx = gx; let vy = gy; let vz = gz;
    let mut corners = String::new();
    for zi in 0..=1u32 {
        for yi in 0..=1u32 {
            for xi in 0..=1u32 {
                let px = min.x + (vx + xi) as f32 * voxel_size;
                let py = min.y + (vy + yi) as f32 * voxel_size;
                let pz = min.z + (vz + zi) as f32 * voxel_size;
                let d = sdf_at(Vec3::new(px, py, pz));
                corners.push_str(&format!(" ({},{},{})={:.2}", vx+xi, vy+yi, vz+zi, d));
            }
        }
    }

    bevy::log::info!(
        "[click] world=({:.1},{:.1},{:.1}) chunk=({},{},{}) grid=({},{},{}) sdf={:.3}{}{}",
        hit_pos.x, hit_pos.y, hit_pos.z,
        cid.x, cid.y, cid.z,
        vx, vy, vz,
        sdf_at(hit_pos),
        boundary_info,
        corners,
    );
}
