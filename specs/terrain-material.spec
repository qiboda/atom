spec: task
name: "Biome 驱动 PBR 材质"
inherits: project
tags: [terrain, material, pbr]
---

## 意图

Phase 2 产出正确的几何体，但所有 biome 渲染为同一白色 StandardMaterial。在做 LOD（Phase 4）之前必须先有 biome 颜色——LOD 会大幅降低三角形数，错失在近距离实现丰富 PBR 的窗口。

利用已存在的 BIOME_VERTEX_ATTRIBUTE 在渲染 shader 中根据 biome 类型选择不同的 albedo/roughness/metallic。

## 已定决策

- **使用 Bevy `MaterialPlugin::<TerrainMaterial>`**，不自定义渲染管线。TerrainMaterial 实现标准 `Material` trait。
- **`BIOME_VERTEX_ATTRIBUTE`**: `MeshVertexAttribute::new("biome", 100, VertexFormat::Uint32)`, 插入为 mesh attribute @location(2)。
- **Biome 编码方案**: compute shader 将 8 个体素角的 biome 类型 (0-5) 用 `pack4xU8` 打包进 2 个 u32。CPU 端 `TerrainChunkVertexInfo::unpack_u32()` 位掩码解包（`0xFF`, `>>8`, `>>16`, `>>24`）。
- **WGSL 端**: `TerrainMaterial` uniform 携带 `biome_colors: array<BiomeColor, 6>`（每 biome 的 base_color + roughness + metallic）。
- **已知退化**: TerrainMaterial 管线当前不渲染（0 vertex shader invocations），回退为白色 StandardMaterial 验证几何体。解决管线 specialize 或 vertex layout 对齐后触发。

## 边界

### 禁止更改

- `assets/shaders/terrain/compute/voxel_vertices.wgsl` — compute 管线 pass 1
- `assets/shaders/terrain/compute/voxel_cross_points.wgsl` — compute 管线 pass 2
- `assets/shaders/terrain/compute/main_mesh_compute_vertices.wgsl` — compute 管线 pass 3
- `assets/shaders/terrain/compute/main_mesh_compute_indices.wgsl` — compute 管线 pass 4
- `crates/atom_terrain/src/chunks/mesh/compute/node.rs` — compute graph node
- `crates/atom_terrain/src/chunks/mesh/compute/buffer.rs` — GPU buffer
- 不新增 Bevy 渲染插件: 复用 `TerrainMaterialPlugin` (MaterialPlugin 封装)
- 不新增 biome 类型: 保持 6 种 (Ocean=0, Forest=1, Desert=2, Plains=3, Mountains=4, Swamp=5)

### 允许更改

- `assets/shaders/terrain/render/terrain_vertex.wgsl` — 顶点着色器 (@location(2) 仅)
- `assets/shaders/terrain/render/terrain_fragment.wgsl` — 片元着色器
- `assets/shaders/terrain/render/terrain_type.wgsl` — 材质类型定义
- `crates/atom_terrain/src/chunks/mesh/materials/` — Rust 端 Material 实现

## 完成条件

场景: 不同 biome 颜色肉眼可辨
  测试: distinct_biome_colors_visible
  假设 TerrainMaterial 管线正常渲染
  并且 场景包含至少 2 种 biome 的 chunk
  当 渲染帧
  那么 两种不同颜色肉眼可见

场景: Forest biome 顶点正确选择 Forest base_color
  测试: forest_biome_vertex_gets_forest_color
  假设 顶点 biome 类型为 Forest (1)
  当 fragment shader 执行
  那么 biome_colors[1].base_color 被选中

场景: Biome 边界颜色平滑过渡
  测试: biome_boundary_vertex_color_smooth
  假设 chunk 处于 biome 边界区域
  当 顶点 biome 由 8 角多数投票确定 (select_dominant_biome)
  那么 边界处无硬台阶——颜色过渡平滑

场景: BIOME_VERTEX_ATTRIBUTE 正确传递到 shader
  测试: biome_vertex_attribute_in_range
  假设 chunk 在 render world 中
  当 BIOME_VERTEX_ATTRIBUTE 从 mesh 传入 fragment shader
  那么 in.vertex_biome 值在 [0, 5] 范围内

场景: CPU 端 biome 解包正确
  测试: unpack_biome_u8s_correctly
  假设 compute shader 输出 voxel_biome[0] 和 voxel_biome[1]
  当 unpack_u32() 解码
  那么 8 个 u8 biome 类型全部在 [0, 5] 范围内
