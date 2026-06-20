# Atom Terrain Engine

基于 Bevy 0.19 的体素平滑地形（GPU Dual Contouring + QEF）。
Bevy API 变更频繁，遇到不确定的 API 先查 `.claude/bevy-kb/migration-index.md`，没有再读 `/data/codes/Bevy` 源码。
架构导航: 高层见 `.claude/intent.lisp`（数据流/管线/约束），符号级见 `cargo doc --open`。

## 文档索引

| 位置 | 内容 |
| `.claude/intent.lisp` | **架构描述符**（数据流/管线/约束，符号导航见 rust-doc） |
| `.claude/AGENTS.md` | **操作协议** → 每次实现任务按 Task List 模板创建勾选框 |
| `.claude/workflow.lisp` | 不变式 + 反模式 + 质量门（精简引用） |
| `.claude/SOUL.md` | Agent 行为规范、依赖规则、测试策略、架构边界 |
| `.claude/ARCHITECTURE.md` | 架构决策记录 (ADR) |
| `.claude/TENSIONS.md` | 摩擦日志 |
| `.claude/plan/` | 跨会话规划 |
| `.claude/bevy-kb/` | **Bevy 知识库**（迁移指南、已验证 API 模式） |
| `.claude/skills/` | **Agent 技能**（agent-spec 工作流） |
| `.claude/AGENTS.md` | 操作协议、缺口检测 |
| `.claude/CAPABILITY-MAP.md` | 当前能力边界 |
| `.claude/APPEND_SYSTEM.md` | 项目代码模式速查（指针索引，指向源码位置） |

## Workspace 当前状态

目前 workspace **仅包含 `crates/atom_terrain`**。其他 crate（atom_render, atom_shader_lib,
atom_ability, atom_layertag, atom_datatables, atom_core, atom_math, atom_renderdoc,
atom_cel_shader, atom_pqef, atom_utils）暂时移出，后续逐步迁入 Bevy 0.19。

**重要**: Bevy debug 构建极慢（~19s 启动，30s+ 出首帧）。运行/测试必须用 `--release`。
验证可用 `cargo run -p atom_terrain --example chunk_loader --release`（超时 30s）。
直接跑二进制需先 `ln -sf $(pwd)/assets target/release/examples/assets`（Bevy 从 exe 目录找 assets）。

## 工作流（强制执行）

非平凡实现任务 MUST 按 `.claude/AGENTS.md` 的 Task List 模板创建勾选框并逐项完成。
跳过任何环节 → commit 视为流程违规。快速任务（typo/import 整理）可豁免 understand→document。


### Edit 工具纪律

- 每次 `edit` MUST 用上一步返回的 `#TAG`，不得跨 edit 复用旧 tag
- 连续 edit ≥ 3 步 → 中间插入 `read` 确认文件状态
- 编辑范围只覆盖变更行；纯增用 `insert`，纯删用 `delete`

- 非平凡逻辑用中文注释；简单辅助函数用英文；Shader 中英文混合
- 公共 API 强制 `#[deny(missing_docs)]` + `///` rust-doc（RFC 1574: Summary/Examples/Panics/Safety）
- 模块组织: `mod.rs` 模式；相关 component/system/resource 分组
- `rustfmt.toml`: Unix 换行, field init shorthand（edition 2024 在 `Cargo.toml` 中配置）
- clippy lint 配置在 `Cargo.toml` 的 `[workspace.lints.clippy]`：`unwrap_used = "warn"`, `too_many_arguments`/`type_complexity`/`collapsible_if` 允许

### 错误处理

- 禁止 `unwrap()` → `expect("原因")`
- `panic!` 允许仅用于硬停止
- 可恢复场景用 `warn!` 记录并跳过
- 不使用 `thiserror`/`anyhow`

## 格式

代码模式速查见 `.claude/APPEND_SYSTEM.md`（待更新，当前以代码为准）。修改代码模式时同步更新该文件。
