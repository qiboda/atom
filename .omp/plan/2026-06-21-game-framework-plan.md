# 游戏框架 MVP 实现计划

> **面向 AI 代理的工作者：** 必需子技能：使用 superpowers:subagent-driven-development（推荐）或 superpowers:executing-plans 逐任务实现此计划。步骤使用复选框（`- [ ]`）语法来跟踪进度。

**目标：** 在 `atom_terrain` crate 中新增 `game/` 模块，提供俯视角摄像机跟随、玩家 WASD 移动、ECS 组件 reflect 注册体系，并通过 example 端到端验证。

**架构：** 三个新文件（`mod.rs`、`camera.rs`、`player.rs`）作为 `game` 模块，一个 `top_down_game.rs` example。`GamePlugin` 统一注册组件和系统。所有游戏组件 `#[reflect(Component)]` 使 BRP 可读写。

**技术栈：** Bevy 0.19 ECS, `bevy_remote` (JSON-RPC over HTTP, 默认 15702 端口), `bevy_pbr`

---

## 文件结构

```
crates/atom_terrain/src/game/
  mod.rs       — GamePlugin: 注册所有组件 + 系统
  camera.rs    — TopDownCamera 组件 + 俯视角跟随系统
  player.rs    — Player/Name/Health/MoveSpeed 组件 + WASD 移动系统
crates/atom_terrain/src/lib.rs
  — 新增 pub mod game;
crates/atom_terrain/examples/top_down_game.rs
  — 端到端 example
```

---

### 任务 1：创建 `game/mod.rs` — GamePlugin 入口

**文件：**
- 创建：`crates/atom_terrain/src/game/mod.rs`

- [ ] **步骤 1：编写 GamePlugin**

```rust
use bevy::prelude::*;

pub mod camera;
pub mod player;

pub use camera::TopDownCamera;
pub use player::{Health, MoveSpeed, Name, Player};

/// 游戏框架根插件，注册所有游戏组件和系统。
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        // 注册组件到类型寄存器（BRP 通过 Reflect 访问）
        app.register_type::<Player>();
        app.register_type::<Name>();
        app.register_type::<Health>();
        app.register_type::<MoveSpeed>();
        app.register_type::<TopDownCamera>();

        // 添加系统
        app.add_systems(Update, (player::player_movement, camera::top_down_camera_follow));
    }
}
```

- [ ] **步骤 2：运行 `cargo check --workspace` 验证编译**

预期：报错 `player` 和 `camera` 模块未定义（因为它们尚未创建）。

---

### 任务 2：创建 `game/player.rs` — 玩家组件与移动

**文件：**
- 创建：`crates/atom_terrain/src/game/player.rs`

- [ ] **步骤 1：编写玩家相关组件和系统**

```rust
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
```

- [ ] **步骤 2：运行 `cargo check --workspace` 验证编译通过**

---

### 任务 3：创建 `game/camera.rs` — 俯视角摄像机系统

**文件：**
- 创建：`crates/atom_terrain/src/game/camera.rs`

- [ ] **步骤 1：编写俯视角摄像机组件和跟随系统**

```rust
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
    player: Query<&Transform, (With<Player>, Changed<Transform>)>,
    mut camera: Query<(&mut Transform, &TopDownCamera), (Without<Player>, With<Camera3d>)>,
) {
    let dt = time.delta_secs();
    let player_pos = match player.iter().last() {
        Some(t) => t.translation,
        None => return,
    };

    for (mut cam_transform, cam_config) in camera.iter_mut() {
        let target_pos = Vec3::new(
            player_pos.x,
            player_pos.y + cam_config.height,
            player_pos.z,
        );
        // 平滑跟随
        cam_transform.translation = cam_transform
            .translation
            .lerp(target_pos, (cam_config.smoothness * dt).min(1.0));
        // 始终正对 -Y 方向（俯视），Z 轴向上
        cam_transform.look_at(player_pos, Vec3::Z);
    }
}
```

- [ ] **步骤 2：运行 `cargo check --workspace` 验证编译通过**

---

### 任务 4：更新 `lib.rs` — 注册 game 模块

**文件：**
- 修改：`crates/atom_terrain/src/lib.rs`

- [ ] **步骤 1：在 lib.rs 中注册 `game` 模块**

在 `pub mod setting;` 之后插入：

```rust
/// 游戏框架系统（俯视角玩家、摄像机、ECS 组件体系）
pub mod game;
```

- [ ] **步骤 2：运行 `cargo check --workspace` 验证编译通过**

---

### 任务 5：创建 `top_down_game.rs` example — 端到端验证

**文件：**
- 创建：`crates/atom_terrain/examples/top_down_game.rs`

- [ ] **步骤 1：编写 example**

```rust
//! 俯视角游戏框架 example。
//!
//! 演示：程序化地形 + 俯视角摄像机 + 玩家 WASD 移动 + BRP 远程访问。
//! BRP HTTP 服务默认监听 127.0.0.1:15702。
//!
//! 启动后控制：
//! - WASD: 移动玩家（蓝色小球体）
//! - 摄像机自动从正上方跟随玩家
//!
//! 远程访问：
//! ```bash
//! curl -X POST http://127.0.0.1:15702 \
//!   -H "Content-Type: application/json" \
//!   -d '{"jsonrpc":"2.0","method":"world.query","id":0,"params":{"data":{"components":["atom_terrain::game::player::Player"]}}}'
//! ```

use bevy::{
    prelude::*,
    remote::{http::RemoteHttpPlugin, RemotePlugin},
};
use atom_terrain::{
    debug::TerrainDebugConfig,
    game::{GamePlugin, Player, Health, MoveSpeed, Name, TopDownCamera},
    GlobalTerrainPlugin,
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

fn setup(mut commands: Commands) {
    // 调试配置：默认关闭线框
    commands.insert_resource(TerrainDebugConfig {
        wireframe: false,
        double_sided: true,
    });

    // 玩家实体（蓝色小球体）
    commands.spawn((
        Player,
        Name("Player".into()),
        Health(100.0),
        MoveSpeed::default(),
        Mesh3d::from(Sphere::new(0.5).mesh().ico(3).unwrap()),
        Material3d::from(Color::srgb(0.2, 0.6, 1.0)),
        Transform::from_xyz(0.0, -20.0, 0.0),
    ));

    // 俯视角摄像机（GamePlugin 会自动跟随玩家）
    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            scale: 1.0,
            ..default()
        }),
        Transform::from_xyz(0.0, -10.0, 0.0).looking_at(Vec3::new(0.0, -20.0, 0.0), Vec3::Z),
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
```

- [ ] **步骤 2：运行 `cargo check --workspace` 验证编译通过**

---

### 任务 6：验证与清理

**文件：**
- 无变更，仅验证

- [ ] **步骤 1：`cargo clippy --workspace`**

预期：零新警告。

- [ ] **步骤 2：`RUSTDOCFLAGS="-Dwarnings" cargo doc --no-deps -p atom_terrain`**

预期：零警告。

- [ ] **步骤 3：运行 example 验证**

```bash
cargo run -p atom_terrain --example top_down_game --release
```

预期：窗口弹出，俯视角正交可见地形 + 蓝色玩家球体，WASD 可行走。

- [ ] **步骤 4：BRP 查询验证（在 example 运行时另开终端）**

```bash
curl -X POST http://127.0.0.1:15702 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"world.query","id":0,"params":{"data":{"components":["atom_terrain::game::player::Player", "atom_terrain::game::player::Name", "atom_terrain::game::player::Health"]}}}'
```

预期：返回 JSON，entities 数组中包含玩家实体，携带 Player/Name/Health 组件。

- [ ] **步骤 5：清理**

确认 example 运行时无调试打印、panic、或未使用的 import。

- [ ] **步骤 6：commit**

```bash
git add \
  crates/atom_terrain/src/game/ \
  crates/atom_terrain/examples/top_down_game.rs \
  crates/atom_terrain/src/lib.rs
git commit -m "feat: add game framework MVP - top-down camera, player movement, reflect components"
```
