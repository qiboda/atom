//! 俯视角游戏框架 example — 完整版本
//!
//! 程序化地形 + 自由相机 + 玩家 + BRP + Agent sidecar。
//!
//! 启动后控制：
//! - 右键拖动: 旋转视角
//! - WASD/QE: 飞行
//! - F1: 切换地形线框
//! - F2: 切换双面渲染
//! - F4: 截图

use atom_terrain::{
    PerChunkTerrainPlugin,
    debug::TerrainDebugConfig,
    game::{GamePlugin, Health, MoveSpeed, Name, Player},
};
use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    prelude::*,
    remote::{RemotePlugin, http::RemoteHttpPlugin},
    render::view::window::screenshot::{save_to_disk, Screenshot},
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    app.add_plugins(PerChunkTerrainPlugin);
    app.add_plugins(FreeCameraPlugin);
    app.add_plugins(GamePlugin);
    app.add_plugins((RemotePlugin::default(), RemoteHttpPlugin::default()));

    app.add_systems(Startup, (setup, start_agent).chain());
    app.add_systems(Update, decorate_agent_entities);
    app.add_systems(Update, take_screenshot);
    app.run();
}

fn take_screenshot(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::F4) {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("SystemTime before UNIX_EPOCH")
            .as_millis();
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(format!("screenshots/terrain-{stamp}.png")));
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(TerrainDebugConfig {
        wireframe: false,
        double_sided: true,
    });

    commands.spawn((
        Player,
        Name("Player".into()),
        Health(100.0),
        MoveSpeed::default(),
        Mesh3d(meshes.add(Sphere::new(1.0).mesh().ico(3).expect("ico(3)"))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.6, 1.0),
            ..default()
        })),
        Transform::from_xyz(0.0, -13.0, 0.0),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, -13.0, 15.0).looking_at(Vec3::new(0.0, -24.0, 0.0), Vec3::Y),
        FreeCamera { walk_speed: 10.0, run_speed: 30.0, ..default() },
    ));

    commands.spawn((
        DirectionalLight { illuminance: 4000.0, ..default() },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::new(0.0, -24.0, 0.0), Vec3::Z),
    ));
}

// ── Agent sidecar 管理 ──

#[derive(Resource)]
struct AgentProcess(std::process::Child);
impl Drop for AgentProcess {
    fn drop(&mut self) {
        info!("[game] Shutting down agent (PID {})...", self.0.id());
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn start_agent(mut commands: Commands) {
    let child = std::process::Command::new("node")
        .args(["--experimental-strip-types", "agent/src/index.ts"])
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .spawn()
        .expect("Failed to start agent sidecar");
    info!("[game] Agent sidecar launched (PID {})", child.id());
    commands.insert_resource(AgentProcess(child));
}

fn decorate_agent_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Name), Without<Mesh3d>>,
) {
    for (entity, name) in &query {
        if name.0 == "NPC" {
            commands.entity(entity).insert((
                Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                MeshMaterial3d(materials.add(Color::srgb(1.0, 0.2, 0.2))),
            ));
        }
    }
}
