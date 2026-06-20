# Draft — gpu-mesh-seams-finish

## Routing
UNCLEAR — "继续" 没有明确产出目标。采用 best-practice 默认方案。

## Status
approved — 2026-06-20

## Current State
- 分支: `feature/gpu-mesh-seams`，工作区干净
- PLAN.md: Phase 0/1/2 全部完成 ✅
- 最近 15 个 commit 集中在 winding、shell 接缝、quad dedup、double-sided 调试
- 最新 commit: `ba2b874 chore: 清理诊断日志、未用函数、调试 example`

## Topology Lock

| # | Component | 描述 |
|---|-----------|------|
| C1 | compact vertex buffer | `qef_solve.wgsl` scatter-write `vertices[compact_index]` + CPU 读 `0..vertex_count`。对齐 dexyfex Reverse Expansion 模式。 |
| C2 | 测试复活 | 解除 `surface_is_contiguous` 的 `#[ignore]`，实现 CPU 端单元测试。`cpu_gpu_noise_parity` 保持 ignore |
| C3 | Shader 清理 | 删除 `open_simplex.wgsl` 中注释掉的 `modf` 代码块(10行)，清理 `fbm.wgsl` 悬空 TODO 注释 |
| C4 | 未用参数清理 | 删除 `_vc`, `_vertex_count`, `_index_count`, `_voxel_size` 四个未用参数 |

## Open Assumptions (Adopted)

| # | Assumption | Default | Rationale | Reversible? |
|---|-----------|---------|-----------|-------------|
| A1 | "继续"的产出 | 完成分支剩余技术债 | 分支名 `feature/gpu-mesh-seams`，自然下一步收尾 | Yes |
| A2 | C2 noise parity | 保持 ignore，更新注释 | CPU OpenSimplex2D ≠ GPU value noise，biome 阶段统一 | Yes |
| A3 | C1 改动方向 | shader scatter-write + CPU compact read | dexyfex 论文 Reverse Expansion：GPU 原子分配 compact_index → scatter-write 顶点到紧凑位置 → CPU 直接读 | Yes |

## Pre-Metus self-grill
- **Q: 改 shader 会不会太激进？** → 只改一行：`vertices[voxel_idx]` → `vertices[vi]`（vi 来自 voxel_alloc）。`qef_solve.wgsl` 已有 voxel_alloc binding（L22），注释本来就写"写入 compacted vertex buffer"。对齐 dexyfex 论文方案。
- **Q: CPU 端真的不需要 remap 了？** → 对。vertex buffer compact 布局后，index buffer 存的就是 compact_index，直接当 tri_indices 用。clamp 后直接 push。

## Pending Action
`write .omo/plans/gpu-mesh-seams-finish.md` — 已完成，待 append todos + Metis + TL;DR
