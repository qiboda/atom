---
name: bevy-api-lookup
description: |
  CRITICAL: Use when encountering uncertain Bevy APIs, types, or migration issues. Triggers on:
  bevy API, how to use, API changed, migration, what type, which method,
  deprecated, removed, renamed, bevy 0.19, bevy version, API reference,
  "这个 API 怎么用", "迁移", "API 变了", "类型不对", "编译错误",
  compute pipeline, bind group, render world, extract resource,
  system ordering, schedule, app builder, render app
---

# Bevy API Lookup

> **Version:** 0.1.0 | **Last Updated:** 2026-06-21

你是 Bevy 0.19 API 的快速检索工具。遇到不确定的 API 时，按以下顺序查找，找到即停。

## 查找顺序

### 1. 查 migration-index（3 秒）

```bash
grep -i "<关键词>" .omp/bevy-kb/migration-index.md
```

`migration-index.md` 是按主题组织的 0.18→0.19 变更速查表。命中 → 读取对应行拿到答案。

### 2. 查 patterns（10 秒）

```bash
grep -i "<关键词>" .omp/bevy-kb/0-19/patterns.md
```

`patterns.md` 是已验证的可编译代码模式。命中 → 直接复用代码片段。

### 3. 查 Bevy 源码（30 秒）

用 `ast_grep` 或 `search` 在 `/data/codes/Bevy/crates/` 中搜索类型定义/使用位置。

```bash
# 找 trait 定义
ast_grep -p "pub trait $TRAIT" /data/codes/Bevy/crates/

# 找方法签名
search pattern="fn $METHOD" /data/codes/Bevy/crates/
```

### 4. 查 Bevy 示例（20 秒）

```bash
find /data/codes/Bevy/examples -name "*.rs" | xargs grep -l "$KEYWORD"
```

### 5. 查 release-notes

```bash
grep -i "<关键词>" .omp/bevy-kb/0-19/release-notes.md
```

## 常见问题速查

| 问题 | 答案 |
|------|------|
| Render Graph API | 已移除，改用 Systems + `Render` schedule |
| Bind Group 创建 | 新 API: `BindGroupLayoutEntries::sequential()` + `render_device.create_bind_group()` |
| ExtractResource | 用 `ExtractResourcePlugin::<T>::default()` 或在 `RenderApp` 手动 `insert_resource` |
| Material 定义 | `impl Material for T`, `AsBindGroup` derive, `MaterialPlugin::<T>` |
| Mesh 创建 | `Mesh::new(PrimitiveTopology, RenderAssetUsages::default())` |
| 系统调度 | `app.add_systems(Update, system)` — 不再用 `add_system` |
| Render World 通信 | `crossbeam::channel` + `Render` schedule system |

## 输出格式

找到答案后，返回:
```
**API**: <类型/函数名>
**来源**: <文件:行号>
**用法**: <代码片段>
**注意**: <迁移陷阱/与旧版差异>
```
