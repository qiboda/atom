//! 俯视角游戏框架 example。
//!
//! 演示：程序化地形 + 俯视角摄像机 + 玩家 WASD 移动 + BRP 远程访问。
//! BRP HTTP 服务默认监听 127.0.0.1:15702。
//!
//! 启动后控制：
//! - WASD: 移动玩家（蓝色球体）
//! - F1: 切换地形线框模式
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
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);

    // 地形系统
    app.add_plugins(GlobalTerrainPlugin::default());

    // 游戏框架
    app.add_plugins(GamePlugin);

    // BRP 远程访问（Agent 入口）
    app.add_plugins((RemotePlugin::default(), RemoteHttpPlugin::default()));

    app.add_systems(Startup, setup);
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
        Mesh3d(meshes.add(Sphere::new(1.0).mesh().ico(3).unwrap())),
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
