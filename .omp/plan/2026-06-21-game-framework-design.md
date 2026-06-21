# 游戏框架 MVP 设计 — 2026-06-21

## 定位

在 GPU 地形管线之上构建游戏框架地基，使玩家能在程序化地形上以俯视角自由移动，并具备 Agent（LLM）可以远程读写的 ECS 组件体系。

此 MVP 不包含 Agent sidecar 进程或召唤 UI——它们是第二阶段的事。

## 架构

```
atom_terrain crate
├── lib.rs
│   ├── compute/          (GPU DC 地形管线，已有)
│   ├── loader/           (动态 chunk 加载，已有)
│   ├── chunk/            (chunk 数据结构，已有)
│   └── game/             [新增] 游戏框架
│       ├── mod.rs        GamePlugin 入口
│       ├── camera.rs     俯视角跟随相机系统
│       └── player.rs     Player 组件 + WASD 移动系统
```

`game` 模块不依赖 terrain 内部实现细节，只通过公开 API（`TerrainObserver`、`TerrainSetting`）与地形系统交互。后续可独立提取为 `atom_core` crate。

## 1. 俯视角摄像机系统 (`camera.rs`)

**行为**:
- `Camera3d` + `Projection::Orthographic`
- 从正上方观察地形，镜头朝向 `-Y`
- 自动跟随带 `Player` 标签的实体
- 保持固定高度偏移 + 平滑插值
- 缩放级别由 `TopDownCamera { height: f32 }` 组件控制，可在游戏运行时调整

**关键约束**:
- 俯视角固定姿态（不旋转），永远向下
- 玩家移动时相机平滑跟随（`lerp`），不硬跟随
- 不影响 `TerrainObserver` 更新（observer 仍用玩家位置触发 chunk 加载）

## 2. 玩家实体 + 移动 (`player.rs`)

**标记组件**:
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
struct Player;
```

**移动**:
- WASD → XZ 平面位移
- `MoveSpeed` 组件控制速度
- 空间: 场景世界空间（Y 轴向下），地形 chunk 在负 Y 区域
- 未做地形表面碰撞——玩家在固定 Y 平面移动（先能走，再考虑爬坡/落地）

**占位几何**: 小球体 `Mesh3d`（PBR 材质，便于看到位置）。

**系统体系**:
- `player_movement`: 读取键盘输入 → 更新 `Transform.translation`
- 运行在 `First` schedule 或 `Update` 中（不与 terrain compute 管线冲突）

## 3. ECS 组件体系 (reg)

所有游戏组件必须在 `app` 中注册 `#[reflect]` + `app.register_type<T>()`，BRP 才能通过 `world.query` / `world.get_components` / `world.mutate_components` 读写。

**MVP 组件集合**:

|组件|字段|反射|用途|
|---|---|---|---|
|`Player`|—|✅|标记玩家实体|
|`Name`|`String`|✅|实体名称|
|`Health`|`f32`|✅|当前血量|
|`MoveSpeed`|`f32`|✅|移动速度|
|`TopDownCamera`|`height: f32`|✅|摄像机高度|

注册方式:
```rust
app.register_type::<Player>();
app.register_type::<Name>();
// ...
```

注册后，Agent 可以:
```json
// 查询所有玩家
{"method": "world.query", "params": {"data": {"components": ["atom_terrain::game::player::Player"]}}}
// 修改玩家血量
{"method": "world.mutate_components", "params": {"entity": 42, "component": "atom_terrain::game::player::Health", "path": "", "value": {"value": 50.0}}}
```

## Example: `top_down_game.rs`

验证端到端：
1. 启动 `DefaultPlugins` + `GlobalTerrainPlugin` + `GamePlugin`
2. 俯视角摄像机，玩家在 (0, -24, 0) 地形表面附近生成
3. WASD 移动玩家，摄像机跟随
4. BRP HTTP 端口 15702 打开，外部可查询/修改组件

## 未包含（后续阶段）

- Agent sidecar 进程
- 召唤 UI / 控制台
- 碰撞检测 / 地形适配
- NPC 系统
- 物品 / 背包

## 完成验收

- [x] `cargo check --workspace` 通过
- [x] `cargo clippy --workspace` 零新警告
- [x] `cargo doc --no-deps -p atom_terrain` 零警告
- [x] example 启动后可见俯视角地形 + 玩家小球 + WASD 可移动
- [x] BRP `world.query` 能查到 Player 实体及其组件
