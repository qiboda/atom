//! 俯视角游戏框架 example。
//!
//! 演示：程序化地形 + 俯视角摄像机 + 玩家 WASD 移动 + BRP 远程访问。
//! BRP HTTP 服务默认监听 127.0.0.1:15702。
//!
//! 启动后控制：
//! - WASD: 移动玩家（蓝色球体）
//! - F1: 切换地形线框模式
//! - F4: 截取当前帧到 screenshots/ 目录
//!
//! 远程访问：
//! ```bash
//! curl -X POST http://127.0.0.1:15702 \
//!   -H "Content-Type: application/json" \
//!   -d '{"jsonrpc":"2.0","method":"world.query","id":0,"params":{"data":{"components":["atom_terrain::game::player::Player"]}}}'
//! ```
use atom_terrain::{
    GlobalTerrainPlugin,
    debug::TerrainDebugConfig,
    game::{GamePlugin, Health, MoveSpeed, Name, Player, TopDownCamera},
};
use bevy::{
    prelude::*,
    remote::{RemotePlugin, http::RemoteHttpPlugin},
    render::view::window::screenshot::{save_to_disk, Screenshot},
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    // 地形系统
    app.add_plugins(GlobalTerrainPlugin);

    // 游戏框架
    app.add_plugins(GamePlugin);

    // BRP 远程访问（Agent 入口）
    app.add_plugins((RemotePlugin::default(), RemoteHttpPlugin::default()));

    app.add_systems(Startup, (setup, start_agent).chain());
    app.add_systems(Update, decorate_agent_entities);
    app.add_systems(Update, take_screenshot);

    // 截图系统
    fn take_screenshot(
        mut commands: Commands,
        input: Res<ButtonInput<KeyCode>>,
    ) {
        if input.just_pressed(KeyCode::F4) {
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            commands
                .spawn(Screenshot::primary_window())
                .observe(save_to_disk(format!("screenshots/terrain-{stamp}.png")));
        }
    }
    app.run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 调试配置：默认关闭线框
    commands.insert_resource(TerrainDebugConfig {
        wireframe: false,
        double_sided: true,
    });

    let terrain_y = -24.0; // 地形表面高度（GPU value noise）

    // 玩家实体（蓝色小球体，放在地形表面）
    commands.spawn((
        Player,
        Name("Player".into()),
        Health(100.0),
        MoveSpeed::default(),
        Mesh3d(meshes.add(Sphere::new(1.0).mesh().ico(3).expect("Sphere ico(3) should succeed"))),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.6, 1.0))),
        Transform::from_xyz(0.0, terrain_y, 0.0),
    ));

    // 俯视角摄像机（GamePlugin 会自动跟随玩家）
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, terrain_y + 10.0, 0.0)
            .looking_at(Vec3::new(0.0, terrain_y, 0.0), Vec3::Z),
        TopDownCamera::default(),
    ));
    // 方向光
    commands.spawn((
        DirectionalLight {
            illuminance: 4000.0,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::new(0.0, -24.0, 0.0), Vec3::Z),
    ));
}

// ── Agent sidecar 管理 ──

/// Agent 子进程句柄，App 退出时自动清理。
#[derive(Resource)]
struct AgentProcess(std::process::Child);
impl Drop for AgentProcess {
    fn drop(&mut self) {
        if let Ok(Some(status)) = self.0.try_wait() {
            info!("[game] Agent already exited with {status}");
        } else {
            info!("[game] Shutting down agent (PID {})...", self.0.id());
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

/// 启动 Agent sidecar 进程。
fn start_agent(mut commands: Commands) {
    let agent_dir = std::env::current_dir()
        .unwrap_or_default()
        .join("agent");
    if !agent_dir.exists() {
        warn!("[game] agent/ directory not found, skipping agent launch");
        return;
    }
    match std::process::Command::new("npx")
        .args(["tsx", "src/index.ts"])
        .current_dir(&agent_dir)
        .spawn()
    {
        Ok(child) => {
            info!("[game] Agent sidecar launched (PID {})", child.id());
            commands.insert_resource(AgentProcess(child));
        }
        Err(e) => {
            warn!("[game] Failed to launch agent: {e}");
        }
    }
}


/// 为 Agent 通过 BRP spawn 的实体自动补上可视 cube。
/// Agent 侧在 spawn 时附带 `Name("NPC")`，本系统匹配并装饰。
fn decorate_agent_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Name), Without<Mesh3d>>,
 ) {
    for (entity, name) in &query {
        if name.0 != "NPC" {
            continue;
        }
        commands.entity(entity).insert((
            Mesh3d(meshes.add(Cuboid::new(1.5, 1.5, 1.5))),
            MeshMaterial3d(materials.add(Color::srgb(1.0, 0.15, 0.15))),
        ));
        info!("[game] Decorated agent entity {entity:?} with cube mesh");
    }
}
