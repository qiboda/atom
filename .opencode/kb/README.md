# Atom 知识库

Bevy 源码是权威参考：编译依赖来自 `qiboda/bevy` 的 `atom-patches` 分支
（`[patch.crates-io]` git 引用，见根 Cargo.toml），本地 checkout 在
`/data/codes/Bevy`（须与分支同步，见 `kb/bevy/README.md`）。本目录记录
已验证的模式、迁移要点和项目知识。

## 结构

```
kb/
├── README.md                    — 本文件
├── ARCHITECTURE.md              — 架构不变量 + ADR
├── bevy/                        — Bevy + 生态（外部依赖）
│   ├── migration-index.md       — 快速查表: "X 变了 → 改成 Y"
│   ├── bsn.md                   — Bevy Scene Notation 参考
│   ├── brp-protocol.md          — BRP 协议参考
│   └── 0-19/
│       ├── release-notes.md     — 0.19 对 atom 相关的变更摘要
│       ├── migration.md         — 0.18→0.19 迁移要点
│       └── patterns.md          — 已验证可编译模式
├── github/                      — GitHub 约定
│   ├── labels.md                — Issue/PR 标签分类（C-/A-/D-/P-/S-）
│   └── comments.md              — 评论规范（永远追加）
└── project/                     — Atom 项目知识
    ├── reflections.md           — 实施后反思 + 历史摩擦归档
    └── game/
        └── README.md            — 游戏系统（待填充）
```

## 使用方式

Bevy API 检索与 shader 审查走 `bevy` skill（`.opencode/skills/bevy/SKILL.md`）——它定义了完整的查找链（migration-index → patterns → Bevy 源码 → 示例）。本文件不重复。

快速入口：

1. Bevy API 问题 → `grep -i "<关键词>" kb/bevy/migration-index.md`
2. 需要代码示例 → `grep -i "<关键词>" kb/bevy/0-19/patterns.md`
3. BRP 协议本身 → 查 `kb/bevy/brp-protocol.md`
4. GitHub 标签/评论约定 → 查 `kb/github/`
5. 以上都没有 → 读 `/data/codes/Bevy` 源码，**然后补一条到对应文件**
