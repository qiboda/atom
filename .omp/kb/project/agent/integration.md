# Agent Integration — Atom

Agent sidecar 与 Bevy 引擎的集成方式、组件路径、decorate 系统。

## Bevy 侧集成

- Agent 由 `start_agent` 系统启动（`npx tsx agent/src/index.ts`）
- Agent 目录相对于 Bevy `current_dir()`（workspace 根）
- PID 存入 `AgentProcess` resource，App 退出时 `Drop` 自动 kill
- Agent 通过 `waitForBevy()` 轮询 `rpc.discover` 等待 Bevy 就绪

## Decorate 系统

BRP `world.spawn_entity` 无法创建 asset handle（`Mesh3d`/`MeshMaterial3d`）。解决方案：

1. Agent spawn entity 时附带 `Name("NPC")` 标识
2. Bevy 侧 `decorate_agent_entities` 系统检测 `Name` 前缀，补 mesh + material

## 组件路径速查

| 组件 | BRP 路径 |
|------|---------|
| Player | `atom_terrain::game::player::Player` |
| Name | `atom_terrain::game::player::Name` |
| Health | `atom_terrain::game::player::Health` |
| MoveSpeed | `atom_terrain::game::player::MoveSpeed` |
| TopDownCamera | `atom_terrain::game::camera::TopDownCamera` |
| Transform | `bevy_transform::components::transform::Transform` |

## 已知限制

- HTTP 轮询 ≥2s 延迟，不适合实时战斗
- Agent 进程崩溃不影响引擎，但需手动重启
