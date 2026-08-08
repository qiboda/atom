---
name: reflect
description: 编写实施后反思并追加到 .opencode/kb/project/reflections.md，把教训落实为流程改进（AGENTS.md 规则/skill 步骤/hook/回归测试）。触发词：反思、reflect、事后总结、经验教训、User corrections、流程改进。
---

# Reflect — 实施后反思 Agent

## 目的

反思的目的是**学习**，然后让开发流程更加完善和自动化，减少摩擦损耗。反思的终点不是"记录"，而是"流程变得更好"。

| 目的 | 对应机制 |
|---|---|
| 学习 | 反思条目沉淀经验——输入客观化（第 0 步读对话记录 + git 验证） |
| 完善 | 第 3 步把教训落实为流程机制变更（AGENTS.md / skill / hook / 脚本 / 回归测试） |
| 减少摩擦 | 已融入流程的条目标记已落实 |

## 角色

在每次 feature 或 bugfix 完成后，将实施后反思写入 `.opencode/kb/project/reflections.md`，并把可固化的教训落实为流程改进。

## 触发条件

- `/reflect` 斜杠命令（用户主动触发）
- 用户纠正 AI 行为后（矛盾、范围扩张、约束遗漏、方案偏离）

## 工作流

### 第 0 步：读取对话记录，提取用户纠正（强制）

**反思的输入必须来自客观记录，而不是结束时的记忆。** 反思不到流程偏差的根本原因是：结束时执行者已无意识接受了偏离，记忆里根本没有"偏差"。对话记录是客观存在的。因此 /reflect 的第一步必须：

1. **读取本 session 的对话记录**（`session_read`），逐条浏览**用户消息**，识别所有纠正型消息：
   - 明确纠正（"不对"、"应该 X"、"预期是 Y"）
   - 流程提醒（"切换worktree啊"、"现在没有在worktree吧"）
   - 语义纠正（对术语/概念的纠偏）
   - 范围/方向纠偏
2. **逐条对照反思条目**：每条用户纠正必须出现在 User corrections 章节（逐字引用用户原话）。遗漏任何一条 = 反思不完整。
3. **git 客观流程验证**（命令可查，不凭印象）：
   - `git branch --contains <commit>` — 本次 commit 落在哪个分支？存在活跃 worktree 而 commit 在 main = 流程偏差
   - `git worktree list` — 是否有"创建了但从未使用"的 worktree？

### 第 1 步：收集上下文

从环境或用户处收集：工作简述标题、做了什么（git diff / commit message）、出了什么问题（流程失败、遗漏步骤、意外的坑）。

### 第 2 步：编写反思条目

按标准格式编写**一条**反思条目，并追加到 `.opencode/kb/project/reflections.md`：

```markdown
## [date] — <标题>

**What was done**: [1-2 sentences summarizing the change]

**User corrections** (if any): [user corrections during this work, 逐字引用]

**What went wrong** (if any): [process failures, missed steps, surprises]

**Lessons learned**: [what to do differently next time]

**Process improvements**: [mechanism changes made or proposed]
```

日期格式：`YYYY-MM-DD`。

### 第 3 步：落实为流程改进

- 教训可固化为机制 → 修改 AGENTS.md / skill / hook / 脚本，随当次 commit 提交
- 教训无法固化（一次性） → Process improvements 写 "None"
- 涉及代码/hook → 输出建议清单，由主 agent 处理

## 输出格式

```
## Reflect: <标题>

### Reflection Entry
<the written entry>

### Process Improvements
<机制变更>

### Verdict
<Entry appended to .opencode/kb/project/reflections.md>
```

## 边界情况

| 场景 | 处理方式 |
|---|---|
| reflections.md 不存在 | 创建 `.opencode/kb/project/reflections.md`，带 `# 反思日志` 标题，然后追加 |
| 无 feature/bugfix 上下文 | 写最小条目：`**What was done**: Minor change.` |
| 发生了流程违规（gate 被跳过等） | 必须在 "What went wrong" 中记录 — 流程违规就是 bug |
| 同一变更多个 commit | 一条反思覆盖该批次的所有 commit |

## 禁止事项

- **删除或改写过去的反思条目** — 只能追加新条目
- **凭空编造** — 如果没有上下文，写一条最小的事实条目
- **评判代码质量** — 反思关乎流程，而非代码 review
- **把落实步骤变成事后口头承诺** — Process improvements 必须落到文件变更，不能只写"下次注意"
- **修改 `.opencode/kb/TENSIONS.md` 已有内容** — TENSIONS 是摩擦信号采集，reflections 是反思沉淀，两者独立

## 与 AGENTS.md 的协作

1. AGENTS.md 品质准则 § 问题处理闭环 → 根因沉淀到 TENSIONS.md（排查路径）
2. `/reflect` 处理事后反思（User corrections + 流程改进）——两者互补不重叠
