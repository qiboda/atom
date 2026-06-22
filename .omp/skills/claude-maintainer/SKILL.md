---
name: claude-maintainer
description: |
  CRITICAL: Use when adding, moving, or reorganizing files under .omp/.
  Triggers on:
  omp setup, .omp management, agent config, context maintenance,
  add to omp, move omp, reorganize omp, omp directory,
  claude setup, .claude management,
  knowledge base, skill definition, spec, plan, workflow,
  "维护 omp", "维护 claude", "加 skill", "放哪里", "怎么分类",
  KB vs skill, 知识库 vs 技能, 分类规则
---

# OMP Maintainer

> **Version:** 0.1.0 | **Last Updated:** 2026-06-21

你是 `.omp/` 目录的维护者。新增/移动/整理任何文件前，必须先按以下规则分类。

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

## 目录职责

```
.omp/
├── AGENTS.md              META   项目上下文 + 文档索引（OMP 自动加载）
├── SOUL.md                META   Agent 行为规范 + 边界
├── ARCHITECTURE.md         KB    架构不变量 + ADR
├── CAPABILITY-MAP.md       KB    当前能力边界
├── TENSIONS.md             KB    摩擦日志
│
├── kb/                     KB    统一知识库
│   ├── README.md               索引与使用流程
│   ├── bevy/                   Bevy + 生态（外部依赖）
│   │   ├── migration-index.md  0.18→0.19 变更速查表
│   │   ├── bsn.md              BSN 参考
│   │   ├── brp-protocol.md     BRP 协议参考
│   │   └── 0-19/
│   │       ├── migration.md    分步迁移指南
│   │       ├── patterns.md     已验证编译模式
│   │       └── release-notes.md
│   ├── pi/                     Pi Agent 框架（外部依赖）
│   │   └── ts-conventions.md   TS 编码规范
│   └── project/                Atom 项目知识
│       ├── agent/
│       │   └── integration.md  Agent sidecar 集成、组件路径
│       └── game/
│           └── README.md       游戏系统（待填充）
│
│
├── skills/                SKILL 可调用能力模块
│   ├── bevy-shader-review/
│   └── claude-maintainer/      元维护（本文）
│
│
├── rules/                 META   OMP 结构化规则
│   └── build-check.mdc         构建检查流程
│
└── commands/              CONFIG 快捷命令
    └── just.md                 Justfile 快捷入口
```

## 新增文件决策树

```
收到新内容
├─ 是流程/工具/可调用操作？
│   └─ → skills/  （写 SKILL.md）
├─ 是静态参考/概念/事实？
│   └─ → kb/ 或 根目录
│       ├─ 是 Bevy / BRP 相关？ → kb/bevy/
│       ├─ 是 Pi / TS 规范？ → kb/pi/
│       ├─ 是项目业务知识？ → kb/project/（agent/ 或 game/）
│       ├─ 是架构/摩擦/能力？ → 根目录（ARCHITECTURE/TENSIONS/CAPABILITY）
├─ 是编码规则/约束？
│   ├─ 简洁铁律（永远适用）→ AGENTS.md 编码规范节
│   └─ 结构化规则（globs/condition）→ rules/
```

## 反模式

| 反模式 | 为何错误 | 正确做法 |
|--------|---------|---------|
| 把参考文档写成 Skill | 静态信息不可调用，Skill 要有输入→流程→输出 | 放入 `kb/` |
| 把流程写成 AGENTS.md 里的句子 | "遇到不确定先查 migration-index" 是可调用流程 | 写成 skill `bevy-api-lookup` |
| Skill 里写大量背景知识 | Skill 变"论文"，行为不清晰 | 背景进 KB，Skill 引用 KB |
| 同概念两处定义 | divergence 风险 | 一个源，别处引用 |
| 过时文件不删不标 | 误导 agent | 标 `⚠️ OBSOLETE` 或直接删除 |


每次改动 `.omp/` 后:
- [ ] 新文件分类正确？（KB / Skill / META）
- [ ] AGENTS.md 文档索引需要更新？
- [ ] 无重复定义？
- [ ] 无过时代码模式引用？
- [ ] Skill 格式正确？（frontmatter + 可执行步骤 + 输出格式）

## 引用

- 分类原则: `memory://root/skills/claude-maintainer/SKILL.md`（本文）
- 已有 skill 格式: `.omp/skills/bevy-api-lookup/SKILL.md`
