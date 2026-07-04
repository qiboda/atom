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
//! - Digit7/3/1: 轴对齐快照视角
//! - Ctrl+Digit: 反向视角

use atom_terrain::{
    PerChunkTerrainPlugin,
    game::{GamePlugin, Name},
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
    app.add_systems(Update, (snap_camera, take_screenshot, decorate_agent_entities));
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

/// Snap camera to axis-aligned view using digit keys (no numpad required)
fn snap_camera(
    input: Res<ButtonInput<KeyCode>>,
    mut q_camera: Single<&mut Transform, With<Camera3d>>,
) {
    let ctrl = input.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let (dir, dist): (Vec3, f32) = if input.just_pressed(KeyCode::KeyY) {
        (if ctrl { Vec3::NEG_Y } else { Vec3::Y }, 20.0)
    } else if input.just_pressed(KeyCode::KeyX) {
        (if ctrl { Vec3::NEG_X } else { Vec3::X }, 20.0)
    } else if input.just_pressed(KeyCode::KeyZ) {
        (if ctrl { Vec3::Z } else { Vec3::NEG_Z }, 20.0)
    } else {
        return;
    };
    **q_camera = Transform::from_translation(dir * dist).looking_at(Vec3::ZERO, Vec3::Y);
}

fn setup(
    mut commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
) {
    // DIAGNOSIS: start double-sided to rule out winding
    commands.insert_resource(atom_terrain::debug::TerrainDebugConfig {
        wireframe: false,
        double_sided: true,
        show_chunk_bounds: true,
        show_world_axes: true,
    });
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        FreeCamera::default(),
    ));
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(0.0, 20.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

// ── Agent sidecar 管理 ──

#[derive(Resource)]
struct AgentProcess(std::process::Child);

impl Drop for AgentProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn start_agent(mut commands: Commands) {
    let child = std::process::Command::new("npx")
        .args(["tsx", "agent/src/index.ts"])
        .spawn()
        .expect("Failed to start agent process");
    commands.insert_resource(AgentProcess(child));
}

fn decorate_agent_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Name), Without<Mesh3d>>,
) {
    for (entity, name) in &query {
        if name.0.starts_with("agent_npc_") {
            let mesh = meshes.add(Cuboid::new(0.5, 1.0, 0.5));
            let mat = materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.8, 0.2),
                ..default()
            });
            commands.entity(entity).insert((Mesh3d(mesh), MeshMaterial3d(mat)));
        }
    }
}
