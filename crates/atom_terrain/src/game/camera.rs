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

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::time::Time;

    fn assert_approx(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-4,
            "expected {a} ≈ {b} but diff = {}",
            (a - b).abs()
        );
    }

    fn camera_app() -> App {
        let mut app = App::new();
        app.init_resource::<Time>();
        app.add_systems(Update, top_down_camera_follow);
        app
    }

    #[test]
    fn top_down_camera_default() {
        let c = TopDownCamera::default();
        assert_eq!(c.height, 10.0);
        assert_eq!(c.smoothness, 5.0);
    }

    #[test]
    fn follows_player_from_above_with_full_lerp() {
        let mut app = camera_app();

        // 玩家 + 相机（均带 Transform）
        app.world_mut().spawn((
            Player,
            Transform::from_translation(Vec3::new(10.0, 0.0, 20.0)),
        ));
        let cam = app
            .world_mut()
            .spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.0, 0.0),
                TopDownCamera {
                    height: 10.0,
                    smoothness: 5.0,
                },
            ))
            .id();

        // smoothness * dt = 5.0 * 1.0 → min(.,1) = 1 → 完全跟随
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f64(1.0));
        app.update();

        let t = app
            .world()
            .entity(cam)
            .get::<Transform>()
            .expect("有 Transform");
        assert_approx(t.translation.x, 10.0);
        assert_approx(t.translation.y, 10.0);
        assert_approx(t.translation.z, 20.0);
    }

    #[test]
    fn no_player_leaves_camera_unchanged() {
        let mut app = camera_app();

        let cam = app
            .world_mut()
            .spawn((
                Camera3d::default(),
                Transform::from_xyz(1.0, 2.0, 3.0),
                TopDownCamera::default(),
            ))
            .id();

        app.update();

        let t = app
            .world()
            .entity(cam)
            .get::<Transform>()
            .expect("有 Transform");
        assert_eq!(t.translation, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn small_dt_lerps_partially() {
        let mut app = camera_app();

        app.world_mut().spawn((
            Player,
            Transform::from_translation(Vec3::new(10.0, 0.0, 0.0)),
        ));
        let cam = app
            .world_mut()
            .spawn((
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.0, 0.0),
                TopDownCamera {
                    height: 10.0,
                    smoothness: 1.0,
                },
            ))
            .id();

        // factor = 1.0 * 0.5 = 0.5 → 半程
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f64(0.5));
        app.update();

        let t = app
            .world()
            .entity(cam)
            .get::<Transform>()
            .expect("有 Transform");
        assert_approx(t.translation.x, 5.0);
        assert_approx(t.translation.y, 5.0);
    }
}
