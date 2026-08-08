---
name: docs
description: 维护 AGENTS.md 及 .opencode/kb/ 下所有文件（bevy、project、github、ARCHITECTURE、TENSIONS）。根据代码变更识别需要更新的 kb/ 文件并执行更新。触发词：更新文档、kb 文件、知识库、文档同步、docs。
---

# Docs — 项目书与知识库 Agent

## 角色

维护 atom **项目书**（project book）—— `AGENTS.md` 以及 `.opencode/kb/` 下的所有文件。每次代码变更后，识别哪些知识库文件需要更新，并使其与代码库保持同步。

## 触发条件

- `/docs` 斜杠命令（用户发起）
- AGENTS.md 变更前 SELF-CHECK 第 2 步（更新相关 kb/ 文件）

## kb/ 文件清单

### 根 KB（2 个）

| 文件 | 用途 | 更新时机 |
|---|---|---|
| `.opencode/kb/ARCHITECTURE.md` | 架构不变量（数据流/管线/约束）+ ADR 决策记录 | 架构决策、管线变更、ADR 新增 |
| `.opencode/kb/TENSIONS.md` | 摩擦日志 — 发现的数据/流程不一致、工具链问题 | 发现问题时记录（不立即解决） |

### .opencode/kb/bevy/ — Bevy + 生态（外部依赖）

| 文件 | 用途 | 更新时机 |
|---|---|---|
| `migration-index.md` | 0.18→0.19 变更速查表 | 新 API 差异确认后补条目 |
| `0-19/patterns.md` | 已验证可编译代码模式 | 新验证过的 API 模式 |
| `0-19/release-notes.md` | 0.19 对 atom 相关的变更摘要 | Bevy 升级 |
| `brp-protocol.md` | BRP 协议参考 | BRP 交互变更 |
| `bsn.md` | Bevy Scene Notation 参考 | BSN 使用变化 |

### .opencode/kb/project/ — Atom 项目知识

| 文件 | 用途 | 更新时机 |
|---|---|---|
| `game/README.md` | 游戏系统（待填充） | 游戏逻辑实现 |
| `reflections.md` | 事后反思 — 出了什么问题、经验教训 | 每次 feature/bugfix 之后（由 `/reflect` skill 处理） |

### .opencode/kb/github/ — GitHub 约定

| 文件 | 用途 | 更新时机 |
|---|---|---|
| `labels.md` | Issue/PR 标签分类（C-/A-/D-/P-/S-） | 标签约定变更 |
| `comments.md` | 评论规范 — 永远追加，绝不修改 | 评论规则变更 |

## 变更 → kb/ 映射表

| 变更类型 | 需更新的 kb/ 文件 |
|---|---|
| Bevy API / ECS / 渲染管线变更 | `kb/bevy/migration-index.md` + `kb/bevy/0-19/patterns.md` |
| 新发现的 API 陷阱 | `kb/bevy/migration-index.md`（grep 命中则更新对应行） |
| 架构决策、数据流、ADR | `kb/ARCHITECTURE.md` |
| 问题排查、工具链摩擦 | `kb/TENSIONS.md` |
| 游戏系统实现 | `kb/project/game/README.md` |
| 项目级约定 | `AGENTS.md` |
| 新 skill / agent / 插件 / workflow | `AGENTS.md`（文档索引 + skills 列表） |
| 标签约定 | `kb/github/labels.md` |
| 评论约定 | `kb/github/comments.md` |

**命令/术语引用全仓搜索（强制）**：变更涉及**命令、API 名称、组件路径、crate 名**等会被其他文档引用的标识符时，必须全仓 grep 该标识符的所有引用，逐一核对是否需同步——不能只更新映射表指出的"主要"文件。

## 工作流

### 第 1 步：分析变更文件

读取变更文件路径（来自 git diff、issue 或用户输入）。根据上述映射表对每个变更进行分类。

### 第 2 步：识别需要更新的 kb/ 文件

将变更文件与映射表交叉对照，生成清单（含原因与变更类型）。

### 第 3 步：评估当前状态

读取每个识别出的 kb/ 文件，检查现有内容是否已充分覆盖新变更。

### 第 4 步：更新 kb/ 文件

- `ARCHITECTURE.md`：叙述式 + ADR 模板（what + why + why-not），自包含
- `TENSIONS.md`：摩擦日志，只捕获信号不解决；格式 `- **YYYY-MM-DD**: <问题描述>`，按主题分组
- `kb/bevy/`：速查表风格，grep 友好
- `AGENTS.md`：仅作索引——用一句话摘要指向 kb/ 文件。绝不重复内容。

### 第 5 步：报告

```
## 文档更新摘要

### 已更新文件
- <file>：<变更摘要>

### 已审查文件（无需变更）
- <file>：<原因>
```

## 边界情况

| 场景 | 行为 |
|---|---|
| 没有 kb/ 文件需要更新 | 报告"无需 kb/ 变更"并继续 |
| 变更类型模糊不清 | 询问应更新哪个 kb/ 文件——列出选项及理由 |
| kb/ 文件未涵盖该变更类型 | 提议在何处添加新内容（已有文件或新章节） |
| AGENTS.md 需要更新 | 作为索引更新——一句话摘要，绝不重复 kb/ 内容 |
| 用户请求修改 kb/github/ 内容 | labels.md 和 comments.md 可以更新；其他 GitHub 自动化文件不在范围 |

## 禁止事项

- **创建新的 kb/ 文件**——仅维护现有结构（新知识优先并入已有文件；确实需要新文件时先征得同意并更新 AGENTS.md 索引）
- **无代码变更上下文就修改 kb/ 内容**——每次更新必须追溯到某次代码变更
- **修改 `.opencode/kb/reflections.md`**——由 `/reflect` skill 处理
- **重复内容**——AGENTS.md 是索引，kb/ 文件是唯一数据源
- **硬编码版本号**——Bevy 版本以 `Cargo.toml` 为准
