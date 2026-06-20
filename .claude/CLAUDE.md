# Atom Terrain Engine

基于 Bevy 0.19 的体素平滑地形（GPU Dual Contouring + QEF）。
架构导航见 `.claude/intent.lisp`。

## 文档索引

| 位置 | 内容 |
| `.claude/intent.lisp` | **架构描述符**（crate→组件→符号导航） |
| `.claude/workflow.lisp` | **开发流程**（understand → research → design → document → implement → verify → review） |
| `.claude/SOUL.md` | Agent 行为规范、依赖规则、测试策略、架构边界 |
| `.claude/ARCHITECTURE.md` | 架构决策记录 (ADR) |
| `.claude/TENSIONS.md` | 摩擦日志 |
| `.plan/` | 跨会话规划 |
| `.claude/AGENTS.md` | 操作协议、缺口检测 |
| `.claude/CAPABILITY-MAP.md` | 当前能力边界 |
| `Justfile` | 开发任务 |
| `specs/*.spec` | 任务 Spec |

## 编码约定

- 非平凡逻辑用中文注释；简单辅助函数用英文；Shader 中英文混合
- 公共 API 强制 `#[deny(missing_docs)]` + `///` rust-doc（RFC 1574: Summary/Examples/Panics）
- 模块组织: `mod.rs` 模式；相关 component/system/resource 分组
- `rustfmt.toml`: Unix 换行, field init shorthand, edition 2024
- `clippy.toml`: `unwrap_used = "warn"`, `too_many_arguments`/`type_complexity`/`collapsible_if` 允许

### 错误处理

- 禁止 `unwrap()` → `expect("原因")`
- `panic!` 允许仅用于硬停止（如 `.atom.project` 找不到）
- 可恢复场景用 `warn!` 记录并跳过
- 不使用 `thiserror`/`anyhow`

### 项目根检测

`.atom.project` 标记文件，`ProjectPaths::root_path()` 从 CWD 向上遍历。

## 格式

代码模式（ECS、Shader 宏、GPU Buffer、LayerTag、TableReader 等）已记录在 `.claude/APPEND_SYSTEM.md` 中，自动注入 system prompt。修改这些模式时同步更新该文件。
