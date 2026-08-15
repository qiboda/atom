# Architecture — Atom Terrain Engine

> 架构不变量（跨子系统约束）+ 架构决策记录 (ADR)。

## 架构不变量

### 数据流

当前代码中存在三条地形管线（按实现顺序）：

1. **Legacy `TerrainPlugin`（4-pass per-chunk，CPU readback）**
   - Main world: `update_grid_chunks` → `ChunkLoadMsg` → `handle_load_requests`
     → `TerrainChunkProcessSender`（crossbeam channel）→ render world
   - Render world: `TerrainChunkMeshComputePlugin`（`gpu.rs`，pass1~4）
     → staging readback → `TerrainChunkMeshSender`（crossbeam channel）→ main world
   - Main world: `handle_mesh_data` → spawn `Mesh3d` + `MeshMaterial3d`
   - 仍保留在代码中，但示例已不再使用。

2. **`PerChunkTerrainPlugin`（32³ per-chunk，6-pass，GPU indirect draw）**
   - Main world: `TerrainObserver` + `ChunkManager` + `ChunkLoadRequest`
     （`ExtractResourcePlugin` 同步到 render world）
   - Render world: `slot_sync_system` 按 `ChunkLoadRequest.wanted` 分配/回收 slot
     → `per_chunk_compute_system`（pass0~5）→ `advance_chunk_states`
   - Render world: `PerChunkRenderPlugin` 直接 `draw_indexed_indirect` 读 GPU buffer，
     **不做 CPU mesh readback**
   - 示例: `top_down_game`

3. **`GlobalTerrainPlugin`（observer-centric 全局 edge graph，6-pass，indirect draw + 可选 readback）**
   - Main world: `TerrainObserver`（`ExtractResourcePlugin` 同步到 render world）
   - Render world: `GlobalTerrainMeshPlugin`（`global_compute.rs` pass0~5）使用 `GlobalMeshPool`
   - Render world: `IndirectTerrainRenderPlugin` 直接 `draw_indexed_indirect`
   - 可选 CPU readback（`GlobalComputeState.readback_enabled`）
     → `TerrainChunkMeshSender` → main world `handle_global_mesh_data`
     （用于碰撞/导航数据）
   - 示例: `chunk_loader`

- **同步**:
  - 新 per-chunk 管线通过 `ExtractResource` 克隆 `ChunkLoadRequest`；
  - 全局管线通过 `ExtractResource` 克隆 `TerrainObserver`；
  - legacy 管线通过 crossbeam channel 传递 chunk 加载/卸载指令与 mesh 回读。
- **回读**: legacy/global 的 GPU sparse → CPU compact + remap 经 crossbeam 回传；
  新 per-chunk 管线不读回 mesh，渲染直接走 GPU indirect draw。

### GPU 管线

- **Per-chunk（`PerChunkTerrainPlugin`）**:
  - `GRID_SIZE = 32`（32³ voxel），`voxel_size = 0.5`；
  - SDF buffer 为 **34³ f32**（N+2 shell）；
  - `MAX_SLOTS = 512`，单 slot 约 **22.3MB**；
  - 6-pass: `sdf_fill` → `edge_detect` → `vertex_alloc` → `qef_solve`
    → `index_build` → `fill_indirect`；
  - 接缝处理: ghost voxel + `neighbor_mask`，QEF 确定性求解。
- **Global（`GlobalTerrainPlugin`）**:
  - `GlobalMeshPool` 默认 `grid_size = 50`（inner 48 + 1-voxel shell 两侧 = 25m 半径）；
  - density grid 为 `(grid_size+1)³ = 51³` 个 f32 采样点；
  - vertex 容量 = `50³`，index 容量 = `50³ × 72`；
  - 6-pass: `sdf_fill` → `edge_detect` → `vertex_alloc` → `qef_solve`
    → `index_build` → `fill_indirect`；
  - 持久 buffer pool（ring/free-list），观察者移动时增量更新；
  - 无 chunk 边界 → 无 shell、无 seam，每条全局 edge 只求解一次。
- **Legacy（`TerrainPlugin`）**:
  - 4-pass: `voxel_vertices` → `voxel_cross_points`
    → `main_mesh_compute_vertices` → `main_mesh_compute_indices`；
  - fixed-slot sparse vertices/indices，CPU readback 后 compact + remap。

### 密度场

- **定义**: `density = y - height_at(x,z)`，positive=air, negative=solid。
- **噪声**: multi-island Voronoi + FBM + coast noise；
  CPU `noise.rs` 与 GPU `sdf_fill.wgsl` 使用同一算法（Voronoi cells + FBM）。
- **真值源**: CPU SDF 是唯一地形真值。GPU mesh 仅视觉；物理/寻路查询 SDF，不查 mesh。

### 跨子系统约束

- **GPU mesh 边界**: 视觉 only；永远不用于物理或寻路
- **异步**: std-only futures；不用 tokio/async-std（Bevy 兼容）

---

## 架构决策记录 (ADR)

### Foundation: Rust + Bevy 0.19

- **日期**: 2026-06
- **状态**: accepted
- **决策**: Rust Edition 2024, Bevy 0.19 ECS + 渲染
- **理由**: Rust 零成本抽象 + GPU compute 安全；Bevy 的 plugin/state/material 体系无需自己造；workspace 模式拆分 crate 边界清晰
- **替代方案**: Godot with Rust GDExtension（GDScript 性能不够 compute）、Unreal Engine with C++（过度重型，不透明管线）、纯 wgpu 无 ECS（需要自己造所有调度和资源管理）
- **后果**:
  - + 编译时安全，GPU buffer 对齐由 encase/bytemuck 保证
  - + Plugin 体系强制关注点分离（render/main world 通信清晰）
  - - Bevy 尚在快速迭代，API 偶尔 breaking
  - - WGSL shader 调试困难（无断点），compute dispatch 需手动验证

> 以下所有子系统的设计（GPU compute pipeline、buffer 管理、数据表系统、技能图）均继承自有代码。不做追溯 ADR——从我们的介入点 forward。

### atom_data: bevy_common_assets 驱动的数据表框架

- **日期**: 2026-08-08
- **状态**: accepted
- **决策**: 新建 `atom_data` crate 作为声明式数据表框架，全面替代 Luban 二进制 datatables 体系（atom_datatables / atom_cfg / atom_macros / atom_luban_lib）。行类型 = `#[derive(DataAsset)]`（仅要求 serde::Deserialize + DataIndexed），表 Asset = 泛型容器 `DataTable<T: DataIndexed>`（`rows: Vec<T>` + 索引容器）；格式由使用方选择，通过 bevy_common_assets 0.17 的 `XxxAssetPlugin::<DataTable<T>>::new(&["ext"])` 注册 loader；索引挂载点 = `AssetEvent::LoadedWithDependencies`（Q8 惰性查询）；目录约定 `assets/datatables/<表类型名>.json`（文件名 = 行类型名，扩展名定格式）。
- **背景**: Luban 生成的二进制 `.bytes` 表不可读、不可手工编辑；访问依赖 `TableReader<T>` SystemParam + 手写 trait 族（MapTable/OneTable/MultiIndexListTable…）；跨表引用是字符串外键 + 运行时手动解析，无声明式支持。
- **选项**:
  - 方案 A (selected): bevy_common_assets 全格式（8 种启用，postcard 不启用）加载 + 自研索引 derive 宏。格式支持广且可扩展，数据源（JSON/RON/TOML/YAML…）对美术/策划可读可编辑；索引声明化（单/多/复合/多值/无五种形态），查询 O(1)。
  - 方案 B: 维持 Luban 二进制体系。不可读、Luban 生成代码重、格式扩展需改生成器，与项目「数据声明化」方向冲突。
  - 方案 C: 自研 loader 仅支持 JSON。实现量可控但违背 Q3「全部格式、格式由使用方选择」；bevy_common_assets 已覆盖全部格式且原生兼容 Bevy 0.19，无重造必要。
- **后果**:
  - + 数据文件全部可读可编辑，热重载（file_watcher）免费获得
  - + 索引声明化：主/次/复合/多值/无索引五种形态由 `#[index(...)]` 属性驱动，查询接口由宏生成（`get`/`get_by_x`/`get_by_pair`/`get_all_by_x`）
  - + `DataTable<T>` 泛型 Asset 单一容器，不按表类型生成独立代码——TypePath 唯一性已 spike 验证（Bevy GenericTypePathCell 按 TypeId 区分，不同 T 实例 path 不同）
  - - 索引构建错误（重复唯一键）在反序列化时以 error 传播，加载失败需使用方监听 `AssetLoadFailedEvent` 感知
  - - 行类型宏生成的查询 trait 是孤儿规则解法（`DataTable<T>` inherent impl 只能在 atom_data 内），调用点需 trait 在 scope（宏与行类型同模块生成，自动满足）
- **关联**: Batch 2 已实现——`DataRegistry` + `#[data_ref]` 跨表引用（见下方 ADR）。

### atom_data Batch 2: DataRegistry + data_ref 跨表引用（已实现）

- **日期**: 2026-08-15
- **状态**: accepted
- **决策**: 在 `atom_data` 中实现 `DataRegistry` 资源与 `#[data_ref(table = "...", key = "...")]` 字段级跨表引用。`DataRegistry` 按行类型 `TypeId` 擦除存储 `DataTable<T>`；`DataRegistryPlugin::register_table::<T>` 为每个行类型注册 `AssetEvent::<DataTable<T>>::LoadedWithDependencies` 同步系统，加载后 clone 进 registry。
- **背景**: 原 ADR 预留 Batch 2 作为“新决策点”；实际实现已完成并被 `atom_ability` 数据访问层使用。
- **选项**:
  - 方案 A (selected): `DataRegistry` + `#[data_ref]` 宏生成 `resolve_{field}` 方法；引用键为字符串，解析失败/非数字键按 skip 语义处理。
  - 方案 B: 继续手写运行时外键解析。重复、易错，与声明式数据表方向不符。
- **后果**:
  - + 跨表引用声明化，查询经 `DataRegistry::table::<T>()` 按 TypeId 取表
  - + 加载同步集中在一处（`LoadedWithDependencies`），调用方无需各自监听资产事件
  - - `DataTable<T>` 需要 `Clone`（registry clone 进表）；行类型需 `'static`
- **关联**: 依赖上方 `atom_data` 框架 ADR；被 `atom_ability::config` 使用。

---

## ADR 模板

```markdown
### [标题]
- **日期**: YYYY-MM-DD
- **状态**: proposed | accepted | deprecated | superseded
- **决策**: 我们决定做什么
- **背景**: 需要解决什么
- **选项**: 考虑过哪些方案
  - 方案 A (selected): 理由
  - 方案 B: 为什么不选
  - 方案 C: 为什么不选
- **后果**: 积极/消极影响
  - + 优势
  - - 代价/限制
- **关联**: 引用其他 ADR #[kebab-id]
```
