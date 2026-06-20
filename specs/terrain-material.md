# Terrain Material — Biome 驱动 PBR 材质

> Phase 3 · 规划中
> 依赖: Phase 2 (biome 密度场形状已完成)

## Intent

Phase 2 产出正确的几何体，但所有 bioma 渲染为同一白色 StandardMaterial。玩家无法从视觉上分辨 Ocean 和 Mountains——整个地形是单调的石膏模型。在添加 LOD（Phase 4）之前必须先有 biome 颜色，因为 LOD 会大幅降低三角形数，错失在近距离实现丰富 PBR 的窗口。

利用已存在的 BIOME_VERTEX_ATTRIBUTE 在渲染 shader 中根据 biome 类型选择不同的 albedo/roughness/metallic。

## Decisions

- **使用 Bevy `MaterialPlugin::<TerrainMaterial>`**，不自定义渲染管线。TerrainMaterial 实现标准 `Material` trait。
- **`BIOME_VERTEX_ATTRIBUTE`**: `MeshVertexAttribute::new("biome", 100, VertexFormat::Uint32)`, 插入为 mesh attribute @location(2)。
- **Biome 编码方案**: compute shader 将 8 个体素角的 biome 类型 (0-5) 用 `pack4xU8` 打包进 2 个 u32。CPU 端 `TerrainChunkVertexInfo::unpack_u32()` 位掩码解包（`0xFF`, `>>8`, `>>16`, `>>24`），`select_dominant_biome()` 投票选主导 biome。这套方案已实现，不换。
- **WGSL 端**: `TerrainMaterial` uniform 携带 `biome_colors: array<BiomeColor, 6>`（每 biome 的 base_color + roughness + metallic）。Fragment shader 用 `in.vertex_biome` 索引该数组。
- **Biome 混合**: vertex 级别混合由 compute shader 的 `select_dominant_biome()` 完成（非 fragment 级三线性）。边界区域自然过渡因为 8 个角的 biome 类型各异且多数投票平滑。
- **已知退化**: TerrainMaterial 管线当前不渲染（0 vertex shader invocations），回退为白色 StandardMaterial 验证几何体。这不在本 spec 范围内——是现有 bug，解决管线 specialize 或 vertex layout 对齐后触发。

## Boundaries

- **NEVER** 修改 compute pipeline 的 4 个 shader (`voxel_vertices.wgsl`, `voxel_cross_points.wgsl`, `main_mesh_compute_vertices.wgsl`, `main_mesh_compute_indices.wgsl`)
- **NEVER** 修改 `crates/atom_terrain/src/chunks/mesh/compute/node.rs` / `buffer.rs` / `pipelines.rs`
- `terrain_vertex.wgsl` 的顶点输入布局只能改 @location(2) (biome 属性)——不能调换 position(@location(0)) 和 normal(@location(1))
- 不新增 Bevy 渲染插件: 必须复用 `TerrainMaterialPlugin` (MaterialPlugin 封装)
- 不新增 biome 类型——保持 6 种 (Ocean=0, Forest=1, Desert=2, Plains=3, Mountains=4, Swamp=5)

## Completion Criteria

| ID | Scenario |
|----|----------|
| C1 | Given TerrainMaterial 管线正常渲染, when 场景包含至少 2 种 biome 的 chunk, then **两种不同颜色肉眼可见** |
| C2 | Given Forest biome 顶点, when fragment shader 执行, then `biome_colors[1].base_color` 被选中 (Forest=1) |
| C3 | Given biome 边界 chunk, when 顶点 biome 是 8 角多数投票结果, then 边界处无硬台阶——颜色过渡平滑（多数投票连续） |
| C4 | Given chunk 在 render world 中, when `BIOME_VERTEX_ATTRIBUTE` 从 mesh 传入 shader, then fragment 中 `in.vertex_biome` 在 [0,5] 范围内 |
| C5 | Given compute shader 输出顶点, when CPU 端读取 `voxel_biome[0]` / `voxel_biome[1]`, then `unpack_u32()` 正确还原 8 个 u8 biome 类型 |
