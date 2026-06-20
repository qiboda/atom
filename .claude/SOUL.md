# Agent Soul — Atom 项目

## How to Be

**先读后动。** 每个任务先读 CLAUDE.md 确认技术栈和约定，再读 AGENTS.md 运行缺口检测。绝不跳过上下文。

**追根溯源。** 遇到设计问题先查现有代码——Bevy 的 ECS 模式、atom_render 的 buffer 抽象、atom_terrain 的状态机。不要凭空设计，复用项目既有模式。

**性能优先。** 这是 GPU compute + 实时渲染项目。hot path 上零分配、零拷贝。Shader 和 GPU buffer 交互用 encase/bytemuck，不引入运行时开销。

**Shader 改动谨慎。** WGSL 编译错误没有 Rust borrow checker 保护。改 compute shader 前确认输入/输出 buffer layout 对齐。改后需实际运行验证——编译通过 ≠ 渲染正确。

**依赖克制。** 标准库→workspace dep→Bevy 生态→新引入（详见 Boundaries）。不加不必要的 dep。

**注释用中文。** Rust 代码的 doc comment 和 Shader 注释混合中英文，非平凡逻辑用中文解释原因。


## Habit: Log Before Fixing
遭遇架构不一致、工具链问题、或流程阻碍时，**先记录到 `.claude/TENSIONS.md` 对应分类下**，再处理。不跳过信号采集直接修复。
分类：GPU 管线 / 数据对齐 / 工具链 / 流程 / 已知退化。

## Testing

按代码层级选择验证策略。Spec 的 Completion Criteria 先行——每个 spec 完成时逐条对账。

| 层级 | 方法 | 工具 |
|------|------|------|
| Spec 验收 | BDD 场景对账 (C1-C6) | spec 文件 + 手动/自动判定 |
| 纯 Rust 数学/逻辑 | 红绿 TDD | `#[test]`, `cargo test` |
| ECS 系统 | 集成测试 (headless App) | `bevy::app::App` 无窗口 |
| GPU compute / Shader | 手动验证 | example + 肉眼 + 记入 TENSIONS.md |

流程：**spec → Completion Criteria → 可自动化？→ (Y) 先测试 (red) 后实现 (green) / (N) 实现后跑 example**。

测试只测行为不测 plumbing——不测默认值、不测内部中间状态。断言逻辑行为而非当前值。

## Boundaries

- 不删除关键性能代码（GPU pipeline、buffer 管理）。
- 不重新设计架构——项目有明确的分层和 Channel 通信模式。
- 密度场/噪声函数是实验区域——改动前先确认当前生效的代码路径。
- 不碰 `.atom.project` 标记文件——它是项目根检测的唯一依据。

## Dependencies

引入新三方库的判断链（按优先级）：

1. **stdlib** — 能用的绝对不引入
2. **workspace 已有 dep** — `crossbeam`、`bytemuck`、`serde` 等直接在 Cargo.toml 里
3. **Bevy 生态已有** — `bevy_flycam`、`bevy-inspector-egui` 等，与引擎深度集成且无需新 Cargo 条目
4. **新引入** — 仅在同时满足：(a) 自实现 > 1 周工作量，(b) 库 ≥1.0 或社区活跃（最近 3 个月有提交），(c) 与现有 dep tree 无冲突

选择理由写入 ARCHITECTURE.md 对应 ADR。
