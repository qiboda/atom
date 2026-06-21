# Agent Knowledge Base — Atom 项目

Agent sidecar（TypeScript + BRP）的知识库。记录 BRP 协议模式、TypeScript 编码约定、已知限制和踩坑记录。

## 索引

| 文件 | 内容 |
|------|------|
| `brp-patterns.md` | BRP 协议模式：world.query / world.spawn_entity / rpc.discover 用法与坑 |
| `ts-conventions.md` | TypeScript 编码规范：模块系统、类型守卫、错误处理、异步模式 |

## 快速参考

- **运行时**: `npx tsx agent/src/index.ts`（Bevy 自动启动，见 `top_down_game.rs` 的 `start_agent`）
- **BRP 端口**: `127.0.0.1:15702`（HTTP JSON-RPC）
- **类型检查**: `npx -p typescript tsc --noEmit`（tsx 运行时不检查类型）
- **依赖**: 仅 `tsx`（devDependency），零运行时 npm 包

## 已知限制

- BRP `world.spawn_entity` 无法创建 `Handle<Mesh>` / `Handle<Material>` → Bevy 侧需 decorate 系统补全
- HTTP 轮询 ≥2s 延迟，不适合实时战斗
- Agent 进程由 Bevy 管理生命周期（`AgentProcess` resource + `Drop` trait）
