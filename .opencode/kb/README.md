# Atom 知识库

Bevy 源码 (`/data/codes/Bevy`) 是权威参考，本目录记录已验证的模式、迁移要点和项目知识。

## 结构

```
kb/
├── README.md                    — 本文件
├── ARCHITECTURE.md              — 架构不变量 + ADR
├── TENSIONS.md                  — 摩擦日志
├── bevy/                        — Bevy + 生态（外部依赖）
│   ├── migration-index.md       — 快速查表: "X 变了 → 改成 Y"
│   ├── bsn.md                   — Bevy Scene Notation 参考
│   ├── brp-protocol.md          — BRP 协议参考
│   └── 0-19/
│       ├── release-notes.md     — 0.19 对 atom 相关的变更摘要
│       ├── migration.md         — 0.18→0.19 迁移要点
│       └── patterns.md          — 已验证可编译模式
└── project/                     — Atom 项目知识
    └── game/
        └── README.md            — 游戏系统（待填充）
```

## 使用方式

1. Bevy API 问题 → `grep -i "<关键词>" kb/bevy/migration-index.md`
2. 需要代码示例 → `grep -i "<关键词>" kb/bevy/0-19/patterns.md`
3. BRP 协议本身 → 查 `kb/bevy/brp-protocol.md`
4. 以上都没有 → 读 `/data/codes/Bevy` 源码，**然后补一条到对应文件**
