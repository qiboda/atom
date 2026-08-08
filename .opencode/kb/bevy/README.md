# Bevy 知识库

本地 Bevy 源码 (`/data/codes/Bevy`) 是权威参考，本目录记录已验证的 API 差异模式和迁移要点。
每次踩坑后增补，避免重复查源码。

## 结构

```
.opencode/kb/bevy/
  README.md              — 本文件
  migration-index.md     — 快速查表: "X 变了 → 改成 Y"
  0-19/
    release-notes.md     — 0.19 对 atom 相关的变更摘要
    migration.md         — 0.18→0.19 迁移要点
    patterns.md          — 已验证的 API 模式（有代码示例）
```

## 使用方式

1. 遇到不认识的 API → 先查 `migration-index.md` 
2. 需要代码示例 → 查 `0-19/patterns.md`
3. 需要背景原因 → 查 `0-19/release-notes.md`
4. 以上都没有 → 读 `/data/codes/Bevy` 源码，**然后补一条到对应文件**
