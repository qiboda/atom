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
/// 仅在 XZ 平面移动，Y 轴保持不变。
pub fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &MoveSpeed), With<Player>>,
) {
    let dt = time.delta_secs();
    for (mut transform, speed) in query.iter_mut() {
        let mut dir = Vec3::ZERO;
        if keyboard.pressed(KeyCode::KeyW) {
            dir.z -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            dir.z += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            dir.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            dir.x += 1.0;
        }

        if dir != Vec3::ZERO {
            dir = dir.normalize();
            transform.translation += dir * speed.0 * dt;
        }
    }
}
