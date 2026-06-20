---
name: bevy-scene-notation
description: |
  CRITICAL: Use for writing and editing Bevy Scene Notation (.bsn) files. Triggers on:
  bsn, bevy scene notation, scene notation, .bsn, 场景描述, ECS 描述,
  write bsn, create bsn, edit bsn, new bsn, bsn format, bsn syntax,
  bevy entity, bevy component, bevy system, bevy resource, bevy schedule,
  system ordering, system chain, system set, app builder,
  "how to write bsn", "bsn 格式", "bsn 语法"
---

# Bevy Scene Notation (BSN)

> **Version:** 0.1.0 | **Last Updated:** 2026-06-21

你是一个 BSN 专家。BSN 是 Bevy ECS 场景的**声明式、diff 友好、可双向转换**的文本记法。

## 设计目标

| 目标 | 说明 |
|------|------|
| **声明式** | 描述「有什么」，不是「怎么建」。对标 Rust `App::build()` 的输出，不描述过程 |
| **diff 友好** | 每行一个事实，修改只动单行。无深层嵌套，无重复声明 |
| **双向** | BSN → Rust: 生成 `app.add_systems(...)` 等 builder 调用；Rust → BSN: 从已有代码反向提取 |
| **可组合** | 插件/模块可以声明自己的 BSN 片段，主场景 `#include` 聚合 |

## 核心语法

### 实体 (Entity)

```
entity <name> {
    component <Type> { <fields> }
    component <Type>         // 单元组件，无字段
    bundle <BundleType>      // 展开 Bundle
    marker <MarkerType>      // 纯标记组件
}
```

示例:
```
entity terrain_observer {
    component TerrainObserver {
        position: Vec3(0.0, -24.0, 0.0),
        force_rebuild: 0,
    }
}
```

### 资源 (Resource)

```
resource <Type> {
    <fields>
}
```

示例:
```
resource TerrainSetting {
    voxel_size: 0.5,
    grid_size: 50,
    height_range: -60.0 .. -10.0,
}
```

### 系统 (System)

```
system <fn_name> {
    schedule: <Schedule>                              // Startup | PreUpdate | Update | PostUpdate | FixedUpdate | Render
    params: [<SystemParam>, ...]                      // Res<T>, Query<...>, Commands, ResMut<T>, Local<Option<Entity>>
    run_if: <condition>                               // 可选
    after: [<system>, ...]                            // 可选，显式排序
    before: [<system>, ...]                           // 可选
    in_set: <SystemSet>                               // 可选
    chain: [<system>, ...]                            // 可选，链式执行
}
```

示例:
```
system update_observer_from_camera {
    schedule: Update
    params: [Query<&Transform, With<Camera3d>>, ResMut<TerrainObserver>]
}

system handle_global_mesh_data {
    schedule: Update
    params: [Commands, Res<TerrainChunkMeshReceiver>, Res<TerrainDebugConfig>, ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>, Local<Option<Entity>>, ResMut<GlobalTerrainMaterial>]
    after: [update_observer_from_camera]
}
```

### 系统集 (SystemSet)

```
system_set <Name> {
    chain: [<system>, ...]                            // 链式执行
    run_if: <condition>                               // 可选
}
```

示例:
```
system_set TerrainSystems {
    chain: [chunk_loader, apply_csg, generate_chunk]
    run_if: in_state(TerrainState::GenerateTerrainMesh)
}
```

### 插件 (Plugin)

```
plugin <Name> {
    entities: [<entity>, ...]
    resources: [<resource>, ...]
    systems: [<system>, ...]
    system_sets: [<system_set>, ...]
    plugins: [<sub_plugin>, ...]                      // 子插件
    event: <EventType>                                // 可选
    state: <StateType>                                // 可选
}
```

示例:
```
plugin GlobalTerrainPlugin {
    resources: [TerrainSetting, TerrainObserver, TerrainDebugConfig]
    systems: [update_observer_from_camera, handle_global_mesh_data, debug_keyboard_toggle, apply_debug_wireframe]
    plugins: [GlobalTerrainMeshPlugin]
}
```

### 包含 (Include)

```
include "path/to/module.bsn"
```

## 项目映射约定

### BSN 字段值 ↔ Rust 类型

| BSN 写法 | Rust 类型 |
|----------|-----------|
| `Vec3(x, y, z)` | `bevy::math::Vec3::new(x, y, z)` |
| `Color::srgb(r, g, b)` | `bevy::color::Color::srgb(r, g, b)` |
| `Color::srgba(r, g, b, a)` | `bevy::color::Color::srgba(r, g, b, a)` |
| `0.5` | `f32` 字面量 |
| `50` | `u32` / `i32` 字面量 |
| `true` | `bool` |
| `"string"` | `&'static str` 或 `String` |
| `a .. b` | `RangeInclusive` |
| `None` / `Some(x)` | `Option` |
| `path::to::Type` | 完整路径的类型引用 |

### 系统参数映射

| BSN | Rust |
|-----|------|
| `Query<&T>` | `Query<&T>` |
| `Query<&mut T>` | `Query<&mut T>` |
| `Query<&T, With<M>>` | `Query<&T, With<M>>` |
| `Res<T>` | `Res<T>` |
| `ResMut<T>` | `ResMut<T>` |
| `Commands` | `Commands` |
| `Local<T>` | `Local<T>` |
| `EventReader<E>` | `EventReader<E>` |
| `EventWriter<E>` | `EventWriter<E>` |

## 使用场景

### 1. 设计阶段：架构文档

在写代码前，用 BSN 描述 ECS 架构。团队 review BSN 而非代码 diff。

```
# terrain-pipeline.bsn — 地形管线的 ECS 架构

plugin TerrainPipeline {
  state: TerrainState

  system_set PreCompute {
    chain: [observe_camera, request_chunks]
    run_if: in_state(TerrainState::GenerateTerrainRegion)
  }

  system_set ComputeDispatch {
    chain: [dispatch_density, dispatch_edges, dispatch_qef, dispatch_indices]
    run_if: resource_changed::<TerrainObserver>
  }

  system_set PostCompute {
    chain: [readback_staging, build_mesh, spawn_mesh_entity]
    run_if: compute_pass_done
  }

  systems: [PreCompute, ComputeDispatch, PostCompute]
}
```

### 2. 代码生成：BSN → Rust

工具可以从 BSN 生成对应的 Rust `App::build()` 代码。生成的代码是模板——框架正确，逻辑留白。

### 3. 反向提取：Rust → BSN

从已有 Rust 代码提取 BSN，用于文档和 review。提取的是**结构**（哪些实体/资源/系统），不提取实现体。

### 4. 验证：BSN vs 实际代码

检查 BSN 与控制台日志中的 system order 是否一致，捕获调度漂移。

## 约束

- BSN 描述**结构**，不描述**实现**（函数体、shader 逻辑）
- BSN 文件不加版本号——跟随仓库一起版本管理
- 每条 BSN 声明应对应 Rust 中唯一的源位置
- 系统参数列表不要求穷举——只列关键参数足够

## 参考

- Bevy 0.19 ECS: `bevy_ecs::schedule`, `bevy_app::App`
- 项目现有架构: `.claude/intent.lisp`（数据流/管线）
