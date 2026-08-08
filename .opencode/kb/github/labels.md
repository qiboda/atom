# 标签

Issue 和 PR 标签遵循 [Bevy](https://github.com/bevyengine/bevy) 分类法，采用基于前缀的分类体系。每个标签由 `<PREFIX>-<Name>` 组成。

## 前缀

| 前缀 | 类别 | 含义 |
|---|---|---|
| **A-** | 领域 | 代码库的哪一部分 |
| **C-** | 类别 | 什么类型的工作 |
| **D-** | 难度 | 有多复杂 |
| **P-** | 优先级 | 有多重要 |
| **S-** | 状态 | issue/PR 的当前状态 |

## A- 领域

| 标签 | 范围 |
|---|---|
| `A-Terrain` | 地形生成、chunk 管理、密度场（`crates/atom_terrain/src/terrain*`、`noise.rs`、`chunk.rs`） |
| `A-Compute` | GPU compute 管线、WGSL shader、buffer 管理（`src/compute/`、`assets/shaders/`） |
| `A-Render` | 渲染管线、mesh、material、相机（`src/mesh/`、`src/loader/`） |
| `A-Game` | 游戏系统、玩家、NPC、camera controller（`src/game/`） |
| `A-Agent` | Agent sidecar、BRP 集成（`agent/`） |
| `A-CI` | CI 工作流、hooks、构建系统（`.github/`、`.githooks/`） |
| `A-Docs` | 项目书（`.opencode/kb/`）、`AGENTS.md`、README |

## C- 类别

| 标签 | 用途 |
|---|---|
| `C-Bug` | 意外或不正确的行为 |
| `C-Feature` | 新功能或能力 |
| `C-Code-Quality` | 重构、难以理解或修改的代码 |
| `C-Performance` | 速度、内存或编译时间改进 |
| `C-Docs` | 文档添加或修正 |
| `C-Question` | 讨论或调研（可能转为功能请求） |
| `C-Chore` | 依赖、CI 脚本、配置或其他非代码变更 |

## D- 难度

| 标签 | 含义 |
|---|---|
| `D-Trivial` | 简单且显而易见的修复 |
| `D-Straightforward` | 方案明确，中等工作量 |
| `D-Complex` | 需要研究、设计或领域专业知识 |

## P- 优先级

| 标签 | 含义 |
|---|---|
| `P-Critical` | 必须立即解决 —— 阻塞关键工作流 |
| `P-High` | 高优先级 |
| `P-Medium` | 中等优先级 |
| `P-Low` | 低优先级 —— 可以等待 |

## S- 状态

| 标签 | 含义 |
|---|---|
| `S-Blocked` | 在其他任务完成之前无法继续 |
| `S-Needs-Investigation` | 在行动前需要进一步调研 |
| `S-CI-Failure` | CI 失败类 issue（手动标记，用于追踪 CI 中断问题） |

## 使用

- 每个 issue 和 PR 必须至少有一个 **A-** 和一个 **C-** 标签。
- **D-**、**P-** 和 **S-** 可选但建议添加。
- PR 继承 issue 的标签；根据需要添加或移除。

```sh
gh issue create --label "C-Bug,A-Terrain"
gh pr create --label "C-Feature,A-Compute"
```
