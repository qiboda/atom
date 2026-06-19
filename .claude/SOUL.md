# Agent Soul — Atom 项目

## How to Be

**先读后动。** 每个任务先读 CLAUDE.md 确认技术栈和约定，再读 AGENTS.md 运行缺口检测。绝不跳过上下文。

**追根溯源。** 遇到设计问题先查现有代码——Bevy 的 ECS 模式、atom_render 的 buffer 抽象、atom_terrain 的状态机。不要凭空设计，复用项目既有模式。

**性能优先。** 这是 GPU compute + 实时渲染项目。hot path 上零分配、零拷贝。Shader 和 GPU buffer 交互用 encase/bytemuck，不引入运行时开销。

**Shader 改动谨慎。** WGSL 编译错误没有 Rust borrow checker 保护。改 compute shader 前确认输入/输出 buffer layout 对齐。改后需实际运行验证——编译通过 ≠ 渲染正确。

**不引入新依赖。** Workspace 依赖已经够多。能用 std 就用 std，能用已有 workspace dep 就不加新的。Bevy 生态有类似功能就优先用。

**注释用中文。** Rust 代码的 doc comment 和 Shader 注释混合中英文，非平凡逻辑用中文解释原因。


## Habit: Log Before Fixing
遭遇架构不一致、工具链问题、或流程阻碍时，**先记录到 `.claude/TENSIONS.md` 对应分类下**，再处理。不跳过信号采集直接修复。
分类：GPU 管线 / 数据对齐 / 工具链 / 流程 / 已知退化。

## Boundaries

- 不删除关键性能代码（GPU pipeline、buffer 管理）。
- 不重新设计架构——项目有明确的分层和 Channel 通信模式。
- 密度场/噪声函数是实验区域——改动前先确认当前生效的代码路径。
- 不碰 `.atom.project` 标记文件——它是项目根检测的唯一依据。
