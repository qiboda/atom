spec: task
name: "单 Biome 地形形状 (MVP)"
inherits: project
tags: [terrain, gpu-compute, mvp]
---

## 意图

MVP 阶段所有 chunk 使用相同的密度场函数，产出一个连续的地形表面。密度场 = y - height_at(x,z)，其中 height_at 由多层 2D value noise 叠加生成。后续迭代替换为 OpenSimplex 并加入 biome 变量。

## 已定决策

- **2D FBM 噪声**: value noise 多层叠加（frequency 0.02/0.08/0.25，amplitude 20/5/1）。产生丘陵状起伏，保证地表连续无碎片。
- **密度场 = y - height_at(x,z)**: 正值=air, 负值=solid。Dual Contouring isosurface = 0。
- **17³ 网格 (16³ voxels)**: (voxel_count+1)³ 计算网格在 chunk 边界处与邻居共享顶点。QEF 确定性保证边界无缝。
- **GPU compute 四 pass** → staging buffer readback → CPU compact+remap → crossbeam → main world Mesh3d:
  1. `voxel_vertices` — 密度场采样到 f32 buffer
  2. `voxel_cross_points` — 边交叉点二分查找 + 法向
  3. `main_mesh_compute_vertices` — QEF 顶点 (Cramer's rule)
  4. `main_mesh_compute_indices` — DC quad 三角索引 (固定 slot, 每 voxel 6 u32)
  5. staging copy → async map → compact (过滤零顶点 + QEF clamp) → remap → Bevy Mesh
- **无 atomics**: 顶点按 voxel index 稀疏存储，索引按固定 slot 写入。CPU 端 compact + remap。
- **QEF clamp**: value noise 尖锐梯度导致 QEF 解超出 voxel 范围，CPU 端 clamp 到 chunk bounds。
- **CPU 端噪声保持 GPU 一致性**: `noise.rs` 提供等价实现，用于碰撞检测和测试。
  ⚠️ **已知偏差**: CPU 端当前为 OpenSimplex2D，GPU 端为 value noise。两者高度不同（GPU height_at(0,0)≈-26，CPU≈-0.14）。待 biome phase 统一为 OpenSimplex。

## 边界

### 禁止更改

- `assets/shaders/terrain/compute/voxel_vertices.wgsl` — density field 入口
- `assets/shaders/terrain/compute/voxel_cross_points.wgsl` — edge cross
- `assets/shaders/terrain/compute/main_mesh_compute_vertices.wgsl` — QEF
- `assets/shaders/terrain/compute/main_mesh_compute_indices.wgsl` — indices
- `crates/atom_terrain/src/compute/gpu.rs` — `init_compute_pipeline` 函数（bind group layout + 4 pipeline 注册）
- bind group layout: @group(0) 6 bindings (uniform + 5 storage)
- `TerrainChunkInfo` 字段顺序和大小（GPU uniform 对齐）

### 允许更改

- 密度场函数 `height_at` 的实现（替换噪声算法或调整参数）
- CPU 端 `noise.rs` 同步更新

## 完成条件

场景: 单个 chunk 产出非空 mesh
  测试: single_chunk_produces_vertices
  状态: ✅ 通过 — 422 verts, 844 tris (chunk at y=-26, release mode)
  假设 chunk 在 surface 高度范围内
  当 compute 四 pass + staging readback 全部完成
  那么 vertex_count > 0
  并且 index_count > 0

场景: 地表连续无碎片
  测试: surface_is_contiguous
  状态: 🔲 待 Phase 2 多 chunk 验证
  假设 两个相邻 chunk 均已生成 mesh
  当 检查共享边界处的顶点
  那么 边界位置差 < voxel_size（QEF 确定性保证）

场景: 顶点在世界空间正确缩放
  测试: vertex_in_world_space
  状态: 🔲 待显式验证（vertex = chunk_min + local * voxel_size，shader 已实现）
  假设 chunk mesh 已生成
  当 读取 mesh 任意顶点位置
  那么 顶点坐标已乘以 voxel_size 并偏移至 chunk_min

场景: CPU 噪声与 GPU 一致
  测试: cpu_gpu_noise_parity
  状态: ⚠️ 已知偏差 — CPU 用 OpenSimplex2D，GPU 用 value noise。两者高度不同。
        待 biome phase 统一为 OpenSimplex 后重新验证。
  假设 给定相同 (x,z) 坐标
  当 CPU noise::height_at(x,z) 与 GPU height_at(x,z) 对比
  那么 差值 < 0.001
