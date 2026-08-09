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
        Query<&Transform, With<Camera3d>>,
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
        if keyboard.pressed(KeyCode::KeyW) {
            dir += forward;
        }
        if keyboard.pressed(KeyCode::KeyS) {
            dir -= forward;
        }
        if keyboard.pressed(KeyCode::KeyA) {
            dir -= right;
        }
        if keyboard.pressed(KeyCode::KeyD) {
            dir += right;
        }

        if dir != Vec3::ZERO {
            dir = dir.normalize();
            transform.translation += dir * speed.0 * dt;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::ButtonInput;
    use bevy::time::Time;

    fn assert_approx(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-5,
            "expected {a} ≈ {b} but diff = {}",
            (a - b).abs()
        );
    }

    fn movement_app() -> App {
        let mut app = App::new();
        app.init_resource::<Time>();
        app.init_resource::<ButtonInput<KeyCode>>();
        app.add_systems(Update, player_movement);
        app
    }

    /// 相机绕 X 轴倾斜 −45°：up 投影到 XZ = (0,0,-1)，right = (1,0,0)。
    fn pitched_camera(app: &mut App) {
        app.world_mut().spawn((
            Camera3d::default(),
            Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
        ));
    }

    fn spawn_player(app: &mut App, pos: Vec3) -> Entity {
        app.world_mut()
            .spawn((
                Player,
                Transform::from_translation(pos),
                MoveSpeed::default(),
            ))
            .id()
    }

    fn run_one_second(app: &mut App) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(std::time::Duration::from_secs_f64(1.0));
        app.update();
    }

    #[test]
    fn move_speed_default_is_10() {
        assert_eq!(MoveSpeed::default().0, 10.0);
    }

    #[test]
    fn w_moves_forward_in_xz_plane() {
        let mut app = movement_app();
        pitched_camera(&mut app);
        let player = spawn_player(&mut app, Vec3::ZERO);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);

        run_one_second(&mut app);

        let t = app
            .world()
            .entity(player)
            .get::<Transform>()
            .expect("有 Transform");
        // forward = (0,0,-1)，speed=10，dt=1s → 沿 -Z 移动 10 米，Y 不变
        assert_approx(t.translation.x, 0.0);
        assert_approx(t.translation.y, 0.0);
        assert_approx(t.translation.z, -10.0);
    }

    #[test]
    fn w_plus_d_moves_diagonally_normalized() {
        let mut app = movement_app();
        pitched_camera(&mut app);
        let player = spawn_player(&mut app, Vec3::ZERO);
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.press(KeyCode::KeyW);
            keys.press(KeyCode::KeyD);
        }

        run_one_second(&mut app);

        let t = app
            .world()
            .entity(player)
            .get::<Transform>()
            .expect("有 Transform");
        // dir = normalize((0,0,-1) + (1,0,0)) = (0.707, 0, -0.707)，位移 10 * dir
        assert_approx(t.translation.x, 7.071067);
        assert_approx(t.translation.z, -7.071067);
        assert_approx(t.translation.y, 0.0);
    }

    #[test]
    fn no_keys_no_movement() {
        let mut app = movement_app();
        pitched_camera(&mut app);
        let player = spawn_player(&mut app, Vec3::new(3.0, 1.0, -4.0));

        run_one_second(&mut app);

        let t = app
            .world()
            .entity(player)
            .get::<Transform>()
            .expect("有 Transform");
        assert_eq!(t.translation, Vec3::new(3.0, 1.0, -4.0));
    }

    #[test]
    fn a_moves_left() {
        let mut app = movement_app();
        pitched_camera(&mut app);
        let player = spawn_player(&mut app, Vec3::ZERO);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyA);

        run_one_second(&mut app);

        let t = app
            .world()
            .entity(player)
            .get::<Transform>()
            .expect("有 Transform");
        // right = (1,0,0)，按 A → dir = -right = (-1,0,0)
        assert_approx(t.translation.x, -10.0);
    }

    #[test]
    fn s_moves_backward() {
        let mut app = movement_app();
        pitched_camera(&mut app);
        let player = spawn_player(&mut app, Vec3::ZERO);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyS);

        run_one_second(&mut app);

        let t = app
            .world()
            .entity(player)
            .get::<Transform>()
            .expect("有 Transform");
        assert_approx(t.translation.z, 10.0);
        assert_approx(t.translation.x, 0.0);
    }

    #[test]
    fn d_moves_right() {
        let mut app = movement_app();
        pitched_camera(&mut app);
        let player = spawn_player(&mut app, Vec3::ZERO);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyD);

        run_one_second(&mut app);

        let t = app
            .world()
            .entity(player)
            .get::<Transform>()
            .expect("有 Transform");
        assert_approx(t.translation.x, 10.0);
        assert_approx(t.translation.z, 0.0);
    }

    #[test]
    fn no_camera_returns_early() {
        let mut app = movement_app();
        let player = spawn_player(&mut app, Vec3::ZERO);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);

        run_one_second(&mut app);

        let t = app
            .world()
            .entity(player)
            .get::<Transform>()
            .expect("有 Transform");
        assert_eq!(t.translation, Vec3::ZERO, "无相机时不做任何移动");
    }

    #[test]
    fn multiple_players_all_move() {
        let mut app = movement_app();
        pitched_camera(&mut app);
        let p1 = spawn_player(&mut app, Vec3::ZERO);
        let p2 = spawn_player(&mut app, Vec3::new(5.0, 2.0, 5.0));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyW);

        run_one_second(&mut app);

        let t1 = app
            .world()
            .entity(p1)
            .get::<Transform>()
            .expect("有 Transform");
        let t2 = app
            .world()
            .entity(p2)
            .get::<Transform>()
            .expect("有 Transform");
        assert_approx(t1.translation.z, -10.0);
        assert_approx(t2.translation.z, -5.0);
        assert_approx(t2.translation.x, 5.0);
    }
}
