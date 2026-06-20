# Terrain LOD — 多分辨率 chunk

> status: in_progress
> phase: 4
> depends: phase-3
↳ SPEC: terrain-material.md (材质系统已完成)

## Intent

当前所有 chunk 使用相同分辨率 (16³ voxels)。64m 外 chunk 的三角形密度与脚下的 chunk 完全一样——浪费 GPU 时间。要做 256m 以上可视距离，需要远距离 chunk 用更少三角形，否则 chunk 数量 × 三角数直接压炸。Phase 3 完成后，先架好 LOD 切换架构，再逐步调整实际计算分辨率。

## Decisions

- **距离分 3 级**: LOD 0 (0-64m, 16³), LOD 1 (64-128m, 8³), LOD 2 (128-256m, 4³)。用 `TerrainChunkLod(pub u32)` 组件存储。
- **`update_chunk_lod` 系统检测变化**: 计算 chunk 中心到 observer 距离，用 `from_distance()` 得到新 LOD。当 LOD 变更时，设置 `TerrainChunkMeshingState::Idle`，触发重新 mesh。避免每帧重新计算——只在 LOD 真正变化时才动作。
- **Phase 4 scope = 架构**: 实现 LOD 组件、切换系统、变化检测。可变 compute 分辨率（per-chunk voxel_num）推迟到后续 phase——需要重构 buffer 分配（当前所有 chunk 共享固定 stride 的 shared buffer）。
- **当前所有 chunk 用 LOD 0 的 max stride**: `SharedStorageBuffer` / `SharedUniformBuffer` 的 stride 基于 LOD 0 的 17³/16³，保证所有 chunk 可用相同 buffer。实际分辨率切换后，挂载不同分辨率的 chunk 需要不同 stride——那是 Phase 4.1 的事。
- **LOD 边界不处理**: Phase 4 不做不同分辨率 chunk 间的焊缝——超出 scope。已知这会可见裂缝，记录在 TENSIONS.md。

## Boundaries

- **NEVER** 修改 shader 文件——Phase 4 只做 CPU 端架构
- **NEVER** 修改 `TerrainChunkMeshBuffers` stride 逻辑——值固定在 LOD 0 max，Phase 4.1 才能动
- **NEVER** 修改 biome 生成器、density_field.wgsl、或材质系统
- 只能改: `crates/atom_terrain/src/chunks/loader/` (loading 系统) + `crates/atom_terrain/src/chunks/mesh/compute/mesh_compute.rs` (状态机) + 新增 LOD 相关类型
- Chunk loader 的 observer 距离检测逻辑不能破坏现有加载/卸载语义——只能增量添加 LOD 判断

## Completion Criteria

| ID | Scenario |
|----|----------|
| C1 | Given observer at (0,0,0), when chunk at (0,0,0) 加载, then `TerrainChunkLod.0 == 0` (距离 < 64m) |
| C2 | Given observer at (0,0,0), when chunk at (100,0,0) 加载, then `TerrainChunkLod.0 > 0` (距离 > 64m) |
| C3 | Given chunk 当前 LOD=0, when observer 移远使距离 > 64m, then LOD 变为 1 且 `TerrainChunkMeshingState` 被设为 `Idle` (触发 meshing) |
| C4 | Given chunk 当前 LOD=1, when LOD 未变, then `update_chunk_lod` 不触发状态转换（无多余 meshing） |
| C5 | Given 任意 chunk, when mesh 生成完成, then `TerrainChunkMeshingState` 正常转移到 `Done`——不卡在 `Meshing` 或 `Computing` |
| C6 | Given chunk 在 LOD=2 范围 (128+ m), when 加载, then chunk 存在且有 `TerrainChunkLod` 组件——系统不崩溃、不 panic |
