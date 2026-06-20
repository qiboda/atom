# Terrain Shape — 基于 Biome 的地形形状

> status: implemented
> phase: 2
> depends: phase-1
↳ ARCHITECTURE.md: Foundation (Rust + Bevy 0.18)

## Intent

Phase 1 产出 biome 纹理但用不上——所有 chunk 的密度场相同，地形是平坦表面。在做材质和 LOD 之前，地形必须已经有了 biome 驱动的形状，否则 PBR 参数贴在毫无起伏的平面上没有意义。

在 compute shader 中，根据 biome 纹理采样切换密度场函数，使 Ocean/Forest/Desert/Plains/Mountains/Swamp 六种 biome 产生可区分的地形。不做这一步，后续所有工作（材质、LOD、CSG）都是建立在无差别的几何体上。

## Decisions

- **2D FBM 噪声而非 3D**: `open_simplex_2d_fbm_with_seed(location.xz, ...)` 产出单一高度值 per (x,z)，保证地表连续无碎片。3D 噪声会产出多值 isosurface（悬浮块），不适合高度场地形。
- **密度场 = `y - height_at(x,z)`**: 当地表以下时密度 < 0（solid），以上时 > 0（air）。Dual Contouring isosurface = 0。
- **6 biome 高度函数定义在 `density_field.wgsl` 的 `get_terrain_noise()` 中**，每个 biome 一个 base + 2D 噪声 × amplitude。
- **Biome 过渡用 2×2 双线性采样** biome 纹理：取 `(u,v)`, `(u+1,v)`, `(u,v+1)`, `(u+1,v+1)` 四个 biome 类型，用小数部分按距离混合它们的高度值。
- **Luma8 纹理值 ×255 恢复 u8 biome 类型**：GPU 将 Luma8 归一化到 [0,1]，`textureLoad` 结果需乘 255 得到 0-5 的真实 biome ID。
- **17³ 网格保持不动**：`(voxel_num+1)³` 计算网格在 chunk 边界处与邻居共享顶点，天然无缝——不碰这个。

## Boundaries

- **NEVER** 修改 Rust compute 框架文件：`crates/atom_terrain/src/chunks/mesh/compute/node.rs`, `mesh_compute.rs`, `buffer.rs`, `pipelines.rs`
- **NEVER** 修改 `assets/shaders/terrain/compute/voxel_utils.wgsl`（索引计算）
- **NEVER** 修改 bind group layout：group(0) 7 bindings + group(1) 3 bindings
- 密度场函数 **只能改** `assets/shaders/terrain/compute/density_field.wgsl`
- `TerrainChunkInfo` 的 `.w` 字段已固定为 `get_terrain_size()` (4096.0)，用于 biome UV 计算——不可改用其他含义

## Completion Criteria

| ID | Scenario |
|----|----------|
| C1 | Given **Ocean biome 区域 chunk**, when mesh 生成, then 顶点表面高度 ≈ -3.0（Ocean base 高度） |
| C2 | Given **Mountains biome 区域 chunk**, when mesh 生成, then 顶点表面高度明显高于 Forest（>6.0） |
| C3 | Given **Forest/Desert 交界 chunk**, when 渲染, then 地形高度平滑过渡，无垂直悬崖（biome 边界连续） |
| C4 | Given **任意 chunk**, when compute shader dispatch, then `vertices_count > 0` 且 `indices_count > 0` |
| C5 | Given **任何非零 density 的 chunk**, when mesh 生成, then 顶点位置已在 world space（乘以 `voxel_size`） |
| C6 | `cargo run -p atom_terrain --example chunk_loader` → 6 种 biome 形状肉眼可辨 |
