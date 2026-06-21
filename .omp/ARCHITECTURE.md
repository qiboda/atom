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
