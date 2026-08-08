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
