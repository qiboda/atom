spec: task
name: "基于 Biome 的地形形状"
inherits: project
tags: [terrain, gpu-compute, biome]
---

## 意图

Phase 1 产出 biome 纹理但用不上——所有 chunk 的密度场相同，地形是平坦表面。在做材质和 LOD 之前，地形必须已经有了 biome 驱动的形状，否则 PBR 参数贴在毫无起伏的平面上没有意义。

在 compute shader 中，根据 biome 纹理采样切换密度场函数，使 Ocean/Forest/Desert/Plains/Mountains/Swamp 六种 biome 产生可区分的地形。

## 已定决策

- **2D FBM 噪声而非 3D**: `open_simplex_2d_fbm_with_seed(location.xz, ...)` 产出单一高度值 per (x,z)，保证地表连续无碎片。3D 噪声会产出多值 isosurface（悬浮块），不适合高度场地形。
- **密度场 = `y - height_at(x,z)`**: 当地表以下时密度 < 0（solid），以上时 > 0（air）。Dual Contouring isosurface = 0。
- **6 biome 高度函数定义在 `density_field.wgsl` 的 `get_terrain_noise()` 中**，每个 biome 一个 base + 2D 噪声 × amplitude。
- **Biome 过渡用 2×2 双线性采样** biome 纹理：取 `(u,v)`, `(u+1,v)`, `(u,v+1)`, `(u+1,v+1)` 四个 biome 类型，用小数部分按距离混合高度值。
- **Luma8 纹理值 ×255 恢复 u8 biome 类型**：GPU 将 Luma8 归一化到 [0,1]，需乘 255 得到 0-5 的真实 biome ID。
- **17³ 网格保持不动**：`(voxel_num+1)³` 计算网格在 chunk 边界处与邻居共享顶点，天然无缝。

## 边界

### 禁止更改

- `crates/atom_terrain/src/chunks/mesh/compute/node.rs` — compute graph node
- `crates/atom_terrain/src/chunks/mesh/compute/mesh_compute.rs` — CPU 编排
- `crates/atom_terrain/src/chunks/mesh/compute/buffer.rs` — GPU buffer 管理
- `crates/atom_terrain/src/chunks/mesh/compute/pipelines.rs` — pipeline 缓存
- `assets/shaders/terrain/compute/voxel_utils.wgsl` — 索引计算
- bind group layout: group(0) 7 bindings + group(1) 3 bindings
- `TerrainChunkInfo` 的 `.w` 字段 = `get_terrain_size()` (4096.0)

### 允许更改

- `assets/shaders/terrain/compute/density_field.wgsl` — 密度场函数

## 完成条件

场景: Ocean biome 区域生成正确的表面高度
  测试: ocean_biome_chunk_surface_height
  假设 biome 纹理在指定坐标点为 Ocean (biome 0)
  当 compute shader 为该 chunk 生成 mesh
  那么 顶点表面高度 ≈ -3.0（Ocean base 高度）

场景: Mountains biome 地形明显高于 Forest
  测试: mountains_biome_higher_than_forest
  假设 存在 Mountains biome chunk
  并且 存在 Forest biome chunk
  当 两个 chunk 都完成 mesh 生成
  那么 Mountains chunk 的平均顶点 Y > Forest chunk 的平均顶点 Y

场景: Biome 边界平滑无断层
  测试: biome_boundary_smooth_transition
  假设 存在 Forest/Desert 交界处的 chunk
  当 compute shader 生成该 chunk 的 mesh
  那么 相邻列顶点间的高度差不超过 biome 边界硬台阶阈值

场景: 所有非空 chunk 产出有效 mesh
  测试: non_empty_chunk_produces_vertices
  假设 密度场在任何点都不恒为正
  当 compute shader dispatch
  那么 vertices_count > 0
  并且 indices_count > 0

场景: 世界空间顶点正确缩放
  测试: vertex_in_world_space
  假设 chunk 已完成 mesh 生成
  当 读取 mesh 任意顶点位置
  那么 顶点坐标已乘以 voxel_size (0.5)
