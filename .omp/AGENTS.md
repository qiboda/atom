# Atom Terrain Engine

基于 Bevy 0.19 的体素平滑地形（GPU Dual Contouring + QEF）。
Bevy API 变更频繁，遇到不确定的 API 先查 `.omp/bevy-kb/migration-index.md`，没有再读 `/data/codes/Bevy` 源码。
架构导航: 高层见 `.omp/intent.lisp`（数据流/管线/约束），符号级见 `cargo doc --open`。

## 文档索引

| 位置 | 内容 |
| `.omp/intent.lisp` | **架构描述符**（数据流/管线/约束，符号导航见 rust-doc） |
| `.omp/workflow.lisp` | 不变式 + 反模式 + 质量门（精简引用） |
| `.omp/SOUL.md` | Agent 行为规范、依赖规则、测试策略、架构边界 |
| `.omp/ARCHITECTURE.md` | 架构决策记录 (ADR) |
| `.omp/TENSIONS.md` | 摩擦日志 |
| `.omp/plan/` | 跨会话规划 |
| `.omp/bevy-kb/` | **Bevy 知识库**（migration、patterns、BSN） |
| `.omp/skills/` | **Agent 技能** |
| `.omp/specs/` | **结构化合同**（project/terrain-shape/terrain-material/math） |
| `.omp/CAPABILITY-MAP.md` | 当前能力边界 |
| `APPEND_SYSTEM.md` | **编码铁律**（自动注入 system prompt） |
| `rules/build-check.mdc` | 构建检查流程 |

## Workspace 当前状态

目前 workspace **仅包含 `crates/atom_terrain`**。其他 crate（atom_render, atom_shader_lib,
atom_ability, atom_layertag, atom_datatables, atom_core, atom_math, atom_renderdoc,
atom_cel_shader, atom_pqef, atom_utils）暂时移出，后续逐步迁入 Bevy 0.19。

**重要**: Bevy debug 构建极慢（~19s 启动，30s+ 出首帧）。运行/测试必须用 `--release`。
验证可用 `cargo run -p atom_terrain --example chunk_loader --release`（超时 30s）。
直接跑二进制需先 `ln -sf $(pwd)/assets target/release/examples/assets`（Bevy 从 exe 目录找 assets）。

## 工作流（强制执行）

非平凡实现任务 MUST 按下方 Task List 模板创建勾选框并逐项完成。
跳过任何环节 → commit 视为流程违规。

### Task List 模板

```
## Task: <简短描述>

- [ ] understand  — 读 spec + bevy-kb + 追踪调用链，能一句话描述要改什么/为什么/验收标准
- [ ] research   — grep 相关符号 + 找可复用模式 + 确认 Bevy API（先查 migration-index.md）
- [ ] design     — 方案写到 .omp/plan/ 下，消歧义到另一个工程师不看对话就能实现
- [ ] document   — 新公共 API 写 rust-doc (RFC 1574)，cargo doc --no-deps 零 warning
- [ ] implement  — 按 design 步进，每步 cargo check，≤3 次编译失败回退 design
- [ ] verify     — cargo clippy + cargo test + agent-spec lifecycle（零 fail）+ 肉眼/example 验证
- [ ] review — commit + 无遗留调试打印/死代码
- [ ] review — 检查 TENSIONS.md: 新发现/架构不一致/工具链问题已记录？
- [ ] review — 检查 bevy-kb: 新 Bevy API 模式或迁移要点已写入？
- [ ] reflect    — 回顾全过程：哪里走弯路？哪些信号被忽略？流程/工具/架构如何改进？产出 ≤5 条 actionable，写入 SESSION-LOG
```

**铁律: NO CHECKBOX UNCHECKED → NO COMMIT.** 跳过任何环节的 commit 视为流程违规。
checklist 不替代思考——它确保每个环节的**退出条件**被显式验证而非默认通过。

### 快速任务豁免

单行修复、typo、import 整理等可在 verify 后直接 commit（跳过 understand/research/design/document，但仍需 implement→verify→review）。

### Edit 工具纪律

- 每次 `edit` MUST 用上一步返回的 `#TAG`，不得跨 edit 复用旧 tag
- 连续 edit ≥ 3 步 → 中间插入 `read` 确认文件状态
- 编辑范围只覆盖变更行；纯增用 `insert`，纯删用 `delete`

## 缺口检测

```
1. 我有这个能力吗？
   ├─ YES → 继续
   └─ NO → 2
2. 我能通过工具构建/安装它吗？
   ├─ YES, 简单 → 构建，继续
   ├─ YES, 复杂 → 提议构建，等待确认
   └─ NO → 3
3. 超出 agent 能解决的范围？
   └─ YES → 明确说明需要什么、为什么做不到、人类可以做什么
```

**永不静默绕过一个缺口。永不假装局限不存在。**

## 摩擦记录

发现数据与系统设计之间的不一致、工具链问题、或流程阻碍时，记录到 `.omp/TENSIONS.md`。不要当场解决——只捕获信号。

```
- YYYY-MM-DD: [category] 描述
```

## 多会话项目协议

当任务跨越多个会话时，维护 `.omp/plan/` 下四个文件：

| 文件 | 用途 |
|------|------|
| `.omp/plan/PLAN.md` | 按阶段组织的勾选框 + 退出标准（每阶段 5-10 项） |
| `.omp/plan/MEMORY.md` | 每个决策 + 理由（防止后续会话无意逆转） |
| `.omp/plan/DRIFT.md` | 规格偏离追踪 |
| `.omp/plan/SESSION-LOG.md` | 每次会话的 handoff note（简报，不是摘要） |

**项目优先级**: 所有扩展和配置放在项目目录下（`.omp/`），不全局安装——clone 即获得完整 agent 能力。

## 能力边界

🔴 始终需要人类的判断:
- 凭证和密钥
- 设计/UX 决策
- 业务逻辑和领域知识
- 模糊需求
- 法律/合规决定

## 编码规范

编码铁律（错误处理/注释/文档/lint）见 `APPEND_SYSTEM.md`（自动注入 system prompt）。
构建检查流程见 `rules/build-check.mdc`。
代码模式以 `crates/atom_terrain/src/` 实际代码为准。
