spec: task
name: "地形 LOD 多分辨率"
inherits: project
tags: [terrain, lod, performance]
---

## 意图

当前所有 chunk 使用相同分辨率 (16³ voxels)。64m 外 chunk 的三角形密度与脚下的完全一样——浪费 GPU 时间。Phase 3 完成后，先架好 LOD 切换架构，再逐步调整实际计算分辨率。

用 `TerrainChunkLod` 组件跟踪每个 chunk 的 LOD 级别，`update_chunk_lod` 系统在 observer 移动时检测 LOD 变化并触发重新 meshing。

## 已定决策

- **距离分 3 级**: LOD 0 (0-64m, 16³), LOD 1 (64-128m, 8³), LOD 2 (128-256m, 4³)。组件 `TerrainChunkLod(pub u32)` 存储。
- **Phase 4 scope = 架构**: 实现 LOD 组件、切换系统、变化检测。可变 compute 分辨率（per-chunk voxel_num）推迟到后续 phase——需要重构 buffer 分配（当前所有 chunk 共享固定 stride）。
- **所有 chunk 用 LOD 0 的 max stride**: SharedStorageBuffer 的 stride 基于 LOD 0 的 17³/16³，保证所有 chunk 可用相同 buffer。
- **LOD 边界不处理**: Phase 4 不做不同分辨率 chunk 间的焊缝——超出 scope。已知这会可见裂缝，记录在 TENSIONS.md。

## 边界

### 禁止更改

- `assets/shaders/` 下所有文件 — Phase 4 只做 CPU 端架构
- `crates/atom_terrain/src/chunks/mesh/compute/buffer.rs` 的 stride 逻辑 — 固定在 LOD 0 max
- `crates/atom_terrain/src/biomes/` — biome 生成器
- `crates/atom_terrain/src/chunks/mesh/materials/` — 材质系统

### 允许更改

- `crates/atom_terrain/src/chunks/loader/` — chunk 加载系统
- `crates/atom_terrain/src/chunks/mesh/compute/mesh_compute.rs` — 状态机
- 新增 LOD 相关类型 (`TerrainChunkLod`, `from_distance()`)

## 完成条件

场景: 近距离 chunk 为 LOD 0
  测试: nearby_chunk_gets_lod_zero
  假设 observer 在 (0, 0, 0)
  当 chunk 在 (0, 0, 0) 加载
  那么 TerrainChunkLod.0 == 0

场景: 远距离 chunk LOD > 0
  测试: distant_chunk_gets_higher_lod
  假设 observer 在 (0, 0, 0)
  当 chunk 在 (100, 0, 0) 加载
  那么 TerrainChunkLod.0 > 0

场景: Observer 移动触发 LOD 变化并重新 meshing
  测试: observer_move_triggers_lod_change_and_remesh
  假设 chunk 当前 LOD=0
  当 observer 移远使距离 > 64m
  那么 LOD 变为 1
  并且 TerrainChunkMeshingState 被设为 Idle

场景: LOD 不变时无多余状态转换
  测试: unchanged_lod_no_state_transition
  假设 chunk 当前 LOD=1
  并且 observer 距离未出当前 LOD 范围
  当 update_chunk_lod 执行
  那么 TerrainChunkMeshingState 不变

场景: Mesh 生成完成后正常流转到 Done
  测试: meshing_completes_to_done_state
  假设 chunk 处于 Meshing 状态
  当 compute dispatch 完成
  那么 TerrainChunkMeshingState == Done

场景: LOD 2 范围 chunk 不崩溃
  测试: lod2_chunk_does_not_panic
  假设 chunk 在 LOD=2 范围 (128m+)
  当 加载
  那么 chunk 实体存在且有 TerrainChunkLod 组件
  并且 无 panic
