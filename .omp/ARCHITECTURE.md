# Architecture Decision Records — Atom

> 记录关键架构决策及其理由。每条记录解决一个问题，说明背景、选项、决策和后果。


## Foundation

### Foundation: Rust + Bevy 0.19

- **日期**: 2026-06
- **状态**: accepted
- **决策**: Rust Edition 2024, Bevy 0.19 ECS + 渲染
- **理由**: Rust 零成本抽象 + GPU compute 安全；Bevy 的 plugin/state/material 体系无需自己造；workspace 模式拆分 crate 边界清晰
- **替代方案**: Godot with Rust GDExtension（GDScript 性能不够 compute）、Unreal Engine with C++（过度重型，不透明管线）、纯 wgpu 无 ECS（需要自己造所有调度和资源管理）
- **后果**:
  - + 编译时安全，GPU buffer 对齐由 encase/bytemuck 保证
  - + Plugin 体系强制关注点分离（render/main world 通信清晰）
  - - Bevy 尚在快速迭代，API 偶尔 breaking
  - - WGSL shader 调试困难（无断点），compute dispatch 需手动验证

> 以下所有子系统的设计（GPU compute pipeline、buffer 管理、数据表系统、技能图）均继承自有代码。不做追溯 ADR——从我们的介入点 forward。

### Agent Sidecar: TypeScript + BRP 远程控制

- **日期**: 2026-06-21
- **状态**: accepted
- **决策**: 使用 TypeScript sidecar 进程 + Bevy Remote Protocol (BRP) 实现游戏逻辑的外部脚本化。Agent 通过 HTTP JSON-RPC 与 Bevy 通信（`world.query`、`world.spawn_entity`），在主循环中轮询玩家位置并触发 NPC 生成。
- **背景**: 需要将游戏逻辑从 Rust 编译时分离，支持热更新和快速迭代。Rust 编译慢（release ~2min），脚本层修改秒级生效。
- **选项**:
  - 方案 A (selected): TypeScript sidecar + BRP — 独立进程，HTTP JSON-RPC 通信，零编译开销，TypeScript 生态成熟
  - 方案 B: Rust 脚本（rhai/mlua）— 嵌入引擎内，减少通信延迟，但脚本 API 需手动绑定，生态弱
  - 方案 C: WASM 组件模型 — 跨语言沙箱，但 Bevy 无原生 WASM host 支持，接入成本高
- **后果**:
  - + TypeScript 修改秒级生效，无需重新编译 Rust
  - + BRP 自动暴露所有 `#[reflect(Component)]` 组件，零绑定代码
  - + Agent 进程独立，崩溃不影响引擎
  - - HTTP 轮询延迟 ≥2s，不适合实时战斗
  - - BRP `world.spawn_entity` 无法创建 asset handle（Mesh/Material），需 Bevy 侧系统补全可视组件
  - - Agent 进程需 Bevy 管理生命周期（spawn/kill）
- **关联**: `agent/src/index.ts`, `crates/atom_terrain/examples/top_down_game.rs`
---

## ADR 模板

```markdown
### [标题]
- **日期**: YYYY-MM-DD
- **状态**: proposed | accepted | deprecated | superseded
- **决策**: 我们决定做什么
- **背景**: 需要解决什么
- **选项**: 考虑过哪些方案
  - 方案 A (selected): 理由
  - 方案 B: 为什么不选
  - 方案 C: 为什么不选
- **后果**: 积极/消极影响
  - + 优势
  - - 代价/限制
- **关联**: 引用其他 ADR #[kebab-id]
```
