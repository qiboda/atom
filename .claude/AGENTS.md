# Agent Self-Awareness

## 操作协议（强制执行）

**每次非平凡实现任务，必须按以下 Task List 模板创建勾选框。**

```
## Task: <简短描述>

- [ ] understand  — 读 spec + bevy-kb + 追踪调用链，能一句话描述要改什么/为什么/验收标准
- [ ] research   — grep 相关符号 + 找可复用模式 + 确认 Bevy API（先查 migration-index.md）
- [ ] design     — 方案写到 .claude/plan/ 下，消歧义到另一个工程师不看对话就能实现
- [ ] document   — 新公共 API 写 rust-doc (RFC 1574)，cargo doc --no-deps 零 warning
- [ ] implement  — 按 design 步进，每步 cargo check，≤3 次编译失败回退 design
- [ ] verify     — cargo clippy + cargo test + 肉眼/example 验证行为
- [ ] review     — commit + 更新 TENSIONS.md + 更新 bevy-kb（如有新 API 模式）
```

**铁律: NO CHECKBOX UNCHECKED → NO COMMIT.** 跳过任何环节的 commit 视为流程违规。

checklist 不替代思考——它确保每个环节的**退出条件**被显式验证而非默认通过。

### 快速任务豁免

单行修复、typo、import 整理等可在 verify 后直接 commit（跳过 understand/research/design/document，但仍需 implement→verify→review）。

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

发现数据与系统设计之间的不一致、工具链问题、或流程阻碍时，记录到 `.claude/TENSIONS.md`。不要当场解决——只捕获信号。

```
- YYYY-MM-DD: [category] 描述
```

## 多会话项目 (.claude/plan/ 协议)

当任务跨越多个会话时，维护 `.claude/plan/` 下四个文件：

| 文件 | 用途 |
|------|------|
| `.claude/plan/PLAN.md` | 按阶段组织的勾选框 + 退出标准（每阶段 5-10 项） |
| `.claude/plan/MEMORY.md` | 每个决策 + 理由（防止后续会话无意逆转） |
| `.claude/plan/DRIFT.md` | 规格偏离追踪 |
| `.claude/plan/SESSION-LOG.md` | 每次会话的 handoff note（简报，不是摘要） |
**项目优先级**: 所有扩展和配置放在项目目录下（`.claude/`），不全局安装——clone 即获得完整 agent 能力。

## 能力边界

🔴 始终需要人类的判断:
- 凭证和密钥
- 设计/UX 决策
- 业务逻辑和领域知识
- 模糊需求
- 法律/合规决定
