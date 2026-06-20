# Atom Terrain Engine

基于 Bevy 0.19 的体素平滑地形（GPU Dual Contouring + QEF）。
Bevy API 变更频繁，遇到不确定的 API 先查 `.claude/bevy-kb/migration-index.md`，没有再读 `/data/codes/Bevy` 源码。
架构导航: 高层见 `.claude/intent.lisp`（数据流/管线/约束），符号级见 `cargo doc --open`。

## 文档索引

| 位置 | 内容 |
| `.claude/intent.lisp` | **架构描述符**（数据流/管线/约束，符号导航见 rust-doc） |
| `.claude/workflow.lisp` | **开发流程**（understand → research → design → document → implement → verify → review） |
| `.claude/SOUL.md` | Agent 行为规范、依赖规则、测试策略、架构边界 |
| `.claude/ARCHITECTURE.md` | 架构决策记录 (ADR) |
| `.claude/TENSIONS.md` | 摩擦日志 |
| `.claude/plan/` | 跨会话规划 |
| `.claude/bevy-kb/` | **Bevy 知识库**（迁移指南、已验证 API 模式） |
| `.claude/AGENTS.md` | 操作协议、缺口检测 |
| `.claude/CAPABILITY-MAP.md` | 当前能力边界 |
| `.claude/APPEND_SYSTEM.md` | 项目代码模式速查（指针索引，指向源码位置） |

## Workspace 当前状态

目前 workspace **仅包含 `crates/atom_terrain`**。其他 crate（atom_render, atom_shader_lib,
atom_ability, atom_layertag, atom_datatables, atom_core, atom_math, atom_renderdoc,
atom_cel_shader）暂时移出，后续逐步迁入 Bevy 0.19。

## 编码约定

- 非平凡逻辑用中文注释；简单辅助函数用英文；Shader 中英文混合
- 公共 API 强制 `#[deny(missing_docs)]` + `///` rust-doc（RFC 1574: Summary/Examples/Panics）
- 模块组织: `mod.rs` 模式；相关 component/system/resource 分组
- `rustfmt.toml`: Unix 换行, field init shorthand, edition 2024
- clippy lint 配置在 `Cargo.toml` 的 `[workspace.lints.clippy]`：`unwrap_used = "warn"`, `too_many_arguments`/`type_complexity`/`collapsible_if` 允许

### 错误处理

- 禁止 `unwrap()` → `expect("原因")`
- `panic!` 允许仅用于硬停止
- 可恢复场景用 `warn!` 记录并跳过
- 不使用 `thiserror`/`anyhow`

## 格式

代码模式速查见 `.claude/APPEND_SYSTEM.md`（待更新，当前以代码为准）。修改代码模式时同步更新该文件。
