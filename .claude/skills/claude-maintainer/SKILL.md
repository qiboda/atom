---
name: claude-maintainer
description: |
  CRITICAL: Use when adding, moving, or reorganizing files under .claude/.
  Triggers on:
  claude setup, .claude management, agent config, context maintenance,
  add to claude, move claude, reorganize claude, claude directory,
  knowledge base, skill definition, spec, plan, workflow,
  "维护 claude", "加 skill", "放哪里", "怎么分类",
  KB vs skill, 知识库 vs 技能, 分类规则
---

# Claude Maintainer

> **Version:** 0.1.0 | **Last Updated:** 2026-06-21

你是 `.claude/` 目录的维护者。新增/移动/整理任何文件前，必须先按以下规则分类。

## 核心分类

> Knowledge Base = "我知道什么"（静态参考，只读）
> Skills = "我会做什么"（可执行流程，可调用）

### 判断标准

> ❓ 它是"信息"，还是"动作流程"？

| 类型 | 放哪里 | 例子 |
|------|--------|------|
| 解释概念 | KB | "ECS 是什么"，"Bevy Scene Notation 语法" |
| 执行流程 | Skill | "如何查 API"，"shader 改前检查" |
| 行为规则 | META（根目录） | "禁止 unwrap"，"commit 前必跑 clippy" |
| 质量门 | META | "Phase-Gate Protocol"，"不变式" |
| 结构化合同 | `specs/` | "terrain-shape.spec"，"qef.spec" |
| 会话记录 | `plan/` | "SESSION-LOG.md"，"DRIFT.md" |

## 目录职责

```
.claude/
├── CLAUDE.md              META   项目总上下文 + 编码约定 + 文档索引
├── AGENTS.md              META   操作协议（Task List 模板）+ 缺口检测
├── SOUL.md                META   Agent 行为规范 + 边界 + spec 生命周期
├── workflow.lisp          META   不变式 + 反模式 + 质量门 + reflect 协议
├── ARCHITECTURE.md         KB    架构决策记录 (ADR)
├── CAPABILITY-MAP.md       KB    当前能力边界
├── TENSIONS.md             KB    摩擦日志（GPU/数据/工具链/流程/退化）
├── intent.lisp             KB    架构描述符（数据流/管线/约束）
│
├── bevy-kb/                KB    Bevy 知识库
│   ├── README.md               索引与使用流程
│   ├── migration-index.md      0.18→0.19 变更速查表
│   ├── bsn.md                  BSN 参考（bsn! 宏语法/用法）
│   └── 0-19/
│       ├── migration.md        分步迁移指南
│       ├── patterns.md         已验证可编译模式
│       └── release-notes.md    0.19 release notes 摘要
│
├── specs/                 SPEC  结构化合同（agent-spec 驱动）
│   ├── project.spec            项目级
│   ├── terrain-shape.spec       功能级
│   ├── terrain-material.spec
│   └── math/                   数学子程序 mini-spec
│       ├── density-sampling.spec
│       └── qef.spec
│
├── skills/                SKILL 可调用能力模块
│   ├── agent-spec-authoring/   agent-spec 工具链
│   ├── agent-spec-estimate/
│   ├── agent-spec-tool-first/
│   ├── bevy-api-lookup/        项目 skills
│   ├── bevy-shader-review/
│   └── claude-maintainer/      元维护（本文）
│
├── plan/                  ARTIFACT 跨会话产物
│   ├── PLAN.md                 阶段勾选框
│   ├── MEMORY.md               决策记忆
│   ├── DRIFT.md                规格偏离
│   ├── SESSION-LOG.md          handoff 简报
│   ├── vision.md               项目方向
│   └── completed/              已完成工作
│
├── rules/                 META   Cursor/enforced 规则
│   ├── build-check.mdc
│   └── code-style.mdc
│
├── commands/              CONFIG 快捷命令
│   ├── check.md
│   ├── clippy.md
│   ├── test.md
│   └── run.md
│
└── settings.local.json    CONFIG 本地权限
```

## 新增文件决策树

```
收到新内容
├─ 是流程/工具/可调用操作？
│   └─ → skills/  （写 SKILL.md）
├─ 是静态参考/概念/事实？
│   └─ → bevy-kb/ 或 根目录
│       ├─ 是 Bevy 相关？ → bevy-kb/
│       ├─ 是架构/摩擦/能力？ → 根目录（ARCHITECTURE/TENSIONS/CAPABILITY）
│       └─ 是项目方向？ → plan/vision.md
├─ 是 spec 合同？
│   └─ → specs/
├─ 是规则/约束？
│   └─ → rules/ 或根目录（CLAUDE/SOUL/AGENTS/workflow）
└─ 是会话记录？
    └─ → plan/
```

## 反模式

| 反模式 | 为何错误 | 正确做法 |
|--------|---------|---------|
| 把参考文档写成 Skill | 静态信息不可调用，Skill 要有输入→流程→输出 | 放入 `bevy-kb/` |
| 把流程写成 CLAUDE.md 里的句子 | "遇到不确定先查 migration-index" 是可调用流程 | 写成 skill `bevy-api-lookup` |
| Skill 里写大量背景知识 | Skill 变"论文"，行为不清晰 | 背景进 KB，Skill 引用 KB |
| 同概念两处定义 | divergence 风险 | 一个源，别处引用 |
| 过时文件不删不标 | 误导 agent | 标 `⚠️ OBSOLETE` 或直接删除 |

## 维护检查清单

每次改动 `.claude/` 后:
- [ ] 新文件分类正确？（KB / Skill / META / SPEC / ARTIFACT）
- [ ] CLAUDE.md 文档索引需要更新？
- [ ] 无重复定义？
- [ ] 无过时代码模式引用？
- [ ] Skill 格式正确？（frontmatter + 可执行步骤 + 输出格式）

## 引用

- 分类原则: `memory://root/skills/claude-maintainer/SKILL.md`（本文）
- 已有 skill 格式: `.claude/skills/agent-spec-authoring/SKILL.md`
- Bevy KB 索引: `.claude/bevy-kb/README.md`
