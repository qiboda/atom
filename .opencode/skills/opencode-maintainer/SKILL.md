---
name: opencode-maintainer
description: |
  CRITICAL: Use when adding, moving, or reorganizing files under .opencode/.
  Triggers on:
  opencode setup, .opencode management, agent config, context maintenance,
  add to opencode, move opencode, reorganize opencode, opencode directory,
  knowledge base, skill definition, spec, plan, workflow,
  "维护 opencode", "维护配置", "加 skill", "放哪里", "怎么分类",
  KB vs skill, 知识库 vs 技能, 分类规则
---

# OpenCode Maintainer

> **Version:** 0.1.0 | **Last Updated:** 2026-06-21

你是 `.opencode/` 目录与根 `AGENTS.md` 的维护者。新增/移动/整理任何文件前，必须先按以下规则分类。

## 核心分类

> Knowledge Base = "我知道什么"（静态参考，只读）
> Skills = "我会做什么"（可执行流程，可调用）
> Rules = "我必须遵守什么"（编码约束，进 AGENTS.md）

### 判断标准

> ❓ 它是"信息"，还是"动作流程"？

| 类型 | 放哪里 | 例子 |
|------|--------|------|
| 解释概念 | KB | "ECS 是什么"，"Bevy Scene Notation 语法" |
| 执行流程 | Skill | "如何查 API"，"shader 改前检查" |
| 行为规则 | META（根 AGENTS.md） | "禁止 unwrap"，"commit 前必跑 clippy" |
| 质量门 | META | "构建检查流程"，"不变式" |

## 目录职责

```
AGENTS.md                    META   项目上下文 + 文档索引（opencode 自动加载）
.opencode/
├── kb/                      KB    统一知识库（含 ARCHITECTURE.md + TENSIONS.md）
│   ├── ARCHITECTURE.md           架构不变量 + ADR
│   ├── TENSIONS.md               摩擦日志
│   ├── README.md                 索引与使用流程
│   ├── bevy/                     Bevy + 生态（外部依赖）
│   │   ├── migration-index.md    0.18→0.19 变更速查表
│   │   ├── bsn.md                BSN 参考
│   │   ├── brp-protocol.md       BRP 协议参考
│   │   └── 0-19/
│   │       ├── migration.md      分步迁移指南
│   │       ├── patterns.md       已验证编译模式
│   │       └── release-notes.md
│   ├── project/                  Atom 项目知识
│   │   └── game/
│   │       └── README.md         游戏系统（待填充）
│   └── github/                   GitHub 约定
│       ├── labels.md             标签分类（C-/A-/D-/P-/S-）
│       └── comments.md           评论规范（永远追加）
│
├── skills/                SKILL 可调用能力模块
│   ├── atom-workflow/            核心工作流（门禁/闭环/提交纪律）
│   ├── issue-workflow/           Issue 生命周期
│   ├── bevy-api-lookup/          Bevy API 检索
│   ├── bevy-shader-review/       Shader 改前检查 + 改后验证
│   ├── test/                     TDD/BDD 测试
│   ├── rustdoc/                  missing_docs 合规
│   ├── docs/                     kb/ 维护
│   ├── reflect/                  事后反思
│   ├── worktree/                 git worktree 管理
│   └── opencode-maintainer/      元维护（本文）
│
├── command/               CONFIG 快捷命令（slash command）
│   └── just.md                   Justfile 快捷入口
│
├── plugins/               PLUGIN 本地插件
│   └── trash-rm.ts               rm → trash-put（防误删）
│
└── opencode.json          CONFIG 配置（permission + plugin 注册）
```

## 新增文件决策树

```
收到新内容
├─ 是流程/工具/可调用操作？
│   └─ → skills/  （写 SKILL.md）
├─ 是静态参考/概念/事实？
│   └─ → .opencode/kb/ 或 根目录
│       ├─ 是 Bevy / BRP 相关？ → kb/bevy/
│       ├─ 是项目业务知识？ → kb/project/（game/）
│       ├─ 是 GitHub 约定？ → kb/github/
│       ├─ 是架构/摩擦？ → kb/（ARCHITECTURE.md / TENSIONS.md）
├─ 是编码规则/约束？
│   ├─ 简洁铁律（永远适用）→ AGENTS.md 编码规范节
│   └─ 结构化规则（globs/condition）→ AGENTS.md 对应章节（opencode 无 .mdc 规则文件）
├─ 是快捷命令？
│   └─ → command/  （description frontmatter + $ARGUMENTS 模板）
```

## 反模式

| 反模式 | 为何错误 | 正确做法 |
|--------|---------|---------|
| 把参考文档写成 Skill | 静态信息不可调用，Skill 要有输入→流程→输出 | 放入 `.opencode/kb/` |
| 把流程写成 AGENTS.md 里的句子 | "遇到不确定先查 migration-index" 是可调用流程 | 写成 skill `bevy-api-lookup` |
| Skill 里写大量背景知识 | Skill 变"论文"，行为不清晰 | 背景进 KB，Skill 引用 KB |
| 同概念两处定义 | divergence 风险 | 一个源，别处引用 |
| 过时文件不删不标 | 误导 agent | 标 `⚠️ OBSOLETE` 或直接删除 |
| 把规则写进 Skill | 规则需常驻上下文，Skill 按需调用 | 编码约束写进根 AGENTS.md |
| Skill frontmatter 缺 description | opencode 会过滤掉无 description 的 skill | `name` + `description` 必填 |


每次改动 `.opencode/` 后:
- [ ] 新文件分类正确？（KB / Skill / META / Command）
- [ ] AGENTS.md 文档索引需要更新？
- [ ] 无重复定义？
- [ ] 无过时代码模式引用？
- [ ] Skill 格式正确？（frontmatter 含 name + description + 可执行步骤 + 输出格式）

## 引用

- 分类原则: `memory://root/skills/opencode-maintainer/SKILL.md`（本文）
- 已有 skill 格式: `.opencode/skills/bevy-api-lookup/SKILL.md`
- opencode 配置规范: `skills/`、`command/`、`agent/` 均为 markdown + frontmatter，见 opencode 官方文档
