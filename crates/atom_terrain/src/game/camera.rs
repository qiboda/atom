use bevy::prelude::*;

use crate::game::player::Player;

/// 俯视角摄像机配置组件。
///
/// 挂载到摄像机实体上，控制观察高度。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct TopDownCamera {
    /// 摄像机与玩家之间的垂直距离（沿 Y 轴）。
    pub height: f32,
    /// 平滑跟随速度系数（越大跟随越硬）。
    pub smoothness: f32,
}

impl Default for TopDownCamera {
    fn default() -> Self {
        Self {
            height: 10.0,
            smoothness: 5.0,
        }
    }
}

/// 俯视角跟随系统，让摄像机始终从正上方跟随 Player 实体。
///
/// 摄像机位置在玩家正上方（Y 正方向），始终看向玩家位置。
/// 使用 lerp 平滑插值避免硬跟随。
pub fn top_down_camera_follow(
    time: Res<Time>,
    player: Query<&Transform, With<Player>>,
    mut camera: Query<(&mut Transform, &TopDownCamera), (Without<Player>, With<Camera3d>)>,
) {
    let dt = time.delta_secs();
    let player_pos = match player.iter().last() {
        Some(t) => t.translation,
        None => return,
    };

    for (mut cam_transform, cam_config) in camera.iter_mut() {
        let target_pos = Vec3::new(player_pos.x, player_pos.y + cam_config.height, player_pos.z);
        // 平滑跟随
        cam_transform.translation = cam_transform
            .translation
            .lerp(target_pos, (cam_config.smoothness * dt).min(1.0));
        // 始终正对 -Y 方向（俯视），Z 轴向上
        cam_transform.look_at(player_pos, Vec3::Z);
    }
}
