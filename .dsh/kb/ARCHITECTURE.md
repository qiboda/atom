# Architecture — Atom Terrain Engine

> 架构不变量（跨子系统约束）+ 架构决策记录 (ADR)。

## 架构不变量

### 数据流

```
Main world:  observer → update_grid_chunks → ChunkLoadMsg (Message)
             → handle_load_requests
             → TerrainChunksToProcess (ExtractResource, main→render each frame)

Render world: per_chunk_compute_system (6-pass state machine)
              → staging buffer → crossbeam channel

Main world:  handle_mesh_data → Mesh3d + MeshMaterial3d
```

- **同步**: TerrainChunksToProcess 通过 ExtractResource 克隆，每帧 idempotent alloc 防止 double-init
- **回读**: GPU sparse → CPU compact + remap via crossbeam

### GPU 管线

- **策略**: per-chunk Dual Contouring，33³ ghost voxel (N+1 overlap)
- **接缝**: ghost voxel + QEF 确定性求解 → 无可见缝隙（<2cm micro-gap 接受）
- **缓冲**: per-slot ring buffer，~25MB/slot，128 slots — chunk 卸载不重置计数器

### 密度场

- **定义**: `density = y - height_at(x,z)`，positive=air, negative=solid
- **噪声**: value noise 3-octave FBM (MVP)；→ OpenSimplex when biome phase begins
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
  - 方案 A (selected): bevy_common_assets 全格式（9 种）加载 + 自研索引 derive 宏。格式支持广且可扩展，数据源（JSON/RON/TOML/YAML…）对美术/策划可读可编辑；索引声明化（单/多/复合/多值/无五种形态），查询 O(1)。
  - 方案 B: 维持 Luban 二进制体系。不可读、Luban 生成代码重、格式扩展需改生成器，与项目「数据声明化」方向冲突。
  - 方案 C: 自研 loader 仅支持 JSON。实现量可控但违背 Q3「全部格式、格式由使用方选择」；bevy_common_assets 已覆盖全部格式且原生兼容 Bevy 0.19，无重造必要。
- **后果**:
  - + 数据文件全部可读可编辑，热重载（file_watcher）免费获得
  - + 索引声明化：主/次/复合/多值/无索引五种形态由 `#[index(...)]` 属性驱动，查询接口由宏生成（`get`/`get_by_x`/`get_by_pair`/`get_all_by_x`）
  - + `DataTable<T>` 泛型 Asset 单一容器，不按表类型生成独立代码——TypePath 唯一性已 spike 验证（Bevy GenericTypePathCell 按 TypeId 区分，不同 T 实例 path 不同）
  - - 索引构建错误（重复唯一键）在反序列化时以 error 传播，加载失败需使用方监听 `AssetLoadFailedEvent` 感知
  - - 行类型宏生成的查询 trait 是孤儿规则解法（`DataTable<T>` inherent impl 只能在 atom_data 内），调用点需 trait 在 scope（宏与行类型同模块生成，自动满足）
- **关联**: 无（新决策点；batch 2 的 DataRegistry + data_ref 跨表引用将在此框架上扩展）

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
