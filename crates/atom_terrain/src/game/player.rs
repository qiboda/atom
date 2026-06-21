use bevy::prelude::*;

/// 玩家标记组件。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Player;

/// 实体名称。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Name(pub String);

/// 实体血量值。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Health(pub f32);

/// 移动速度（世界单位/秒）。
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct MoveSpeed(pub f32);

impl Default for MoveSpeed {
    fn default() -> Self {
        Self(10.0)
    }
}

/// WASD 移动系统，每帧读取键盘输入并更新玩家位置。
///
/// 移动方向相对于摄像机视角：W=屏幕上方、S=屏幕下方、A=屏幕左方、D=屏幕右方。
/// 仅在 XZ 平面移动，Y 轴保持不变。
pub fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut params: ParamSet<(
        Query<&Transform, (With<Camera3d>, With<crate::game::camera::TopDownCamera>)>,
        Query<(&mut Transform, &MoveSpeed), With<Player>>,
    )>,
) {
    let dt = time.delta_secs();
    let (forward, right) = {
        let cam = params.p0();
        let t = match cam.single() {
            Ok(t) => t,
            Err(_) => return,
        };
        // 摄像机局部轴在 XZ 平面上的投影 → 屏幕方向
        let forward = Vec3::new(t.up().x, 0.0, t.up().z).normalize_or_zero();
        let right = Vec3::new(t.right().x, 0.0, t.right().z).normalize_or_zero();
        (forward, right)
    };

    for (mut transform, speed) in params.p1().iter_mut() {
        let mut dir = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyW) { dir += forward; }
        if keyboard.pressed(KeyCode::KeyS) { dir -= forward; }
        if keyboard.pressed(KeyCode::KeyA) { dir -= right; }
        if keyboard.pressed(KeyCode::KeyD) { dir += right; }

        if dir != Vec3::ZERO {
            dir = dir.normalize();
            transform.translation += dir * speed.0 * dt;
        }
    }
}
