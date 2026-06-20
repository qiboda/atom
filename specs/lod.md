# Spec: 地形 LOD

> Phase 4 · 规划中
> 依赖: Phase 3 (材质系统完成)


## 目标

远距离 Chunk 使用更粗粒度的 mesh，支撑更大可视范围。

## 规则

LOD 层级由 Chunk 到相机距离决定：

| 距离 | Voxel 大小 | Chunk 分辨率 |
|------|-----------|-------------|
| 0-64m | 0.5m (base) | 16³ |
| 64-128m | 1.0m (2×) | 8³ |
| 128-256m | 2.0m (4×) | 4³ |
| 256m+ | 不可见 | — |

## Chunk 接缝

相邻不同 LOD 的接缝必须无缝。利用现有 17³ 计算网格（外扩一个 voxel）确保边界顶点在 shared edge 上。

## 验收

- 远距离 Chunk 三角形数明显减少
- 接缝不可见
- 整体可视距离 ≥ 256m

## 约束

- LOD 切换在 Chunk 重新 mesh 时触发
- 复用 `TerrainMaterialUniform.lod` 字段
- Compute shader 需支持可变分辨率
