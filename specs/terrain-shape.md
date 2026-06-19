# Spec: 基于 Biome 的地形形状

> Phase 2 · 进行中
> 依赖: Phase 1 (biome 纹理已生成)

## 现状

- Density compute pipeline 已就绪: `assets/shaders/terrain/compute/` 下 4 个 pass
- Pass 1 (density values) 当前使用硬编码简单密度场，噪声调用已注释
- Biome 纹理作为 `TerrainRegionInfo.biome_image` (Handle<Image>) 已可 GPU 采样
- `TerrainRegionGpuConfig.size_per_pixel` 已 extract 到 render world
- GPU compute 使用 17³ 计算网格 (`get_voxel_count_in_compute()`)

## 目标

在 `compute_voxel_vertex_values` shader pass 中，根据 biome 纹理采样选择密度场函数，使 6 种 biome 产生不同地形形状。

## 规则

每个 biome 定义自己的密度场函数：

| Biome | 密度场特征 | 实现 |
|-------|----------|------|
| Ocean | 低于海平面，平坦底面 | `f = y + 8.0` |
| Plains | 低起伏，开阔 | `f = y - 2.0 + simplex_2d(x,z) * 2` |
| Forest | 中等起伏，有自然坡度 | `f = y - 3.0 + fbm_2d(x,z, 3) * 4` |
| Desert | 平坦，有沙丘波纹 | `f = y - 1.0 + abs(simplex_2d(x,z)) * 3` |
| Mountains | 剧烈起伏，尖锐山峰 | `f = y - 10.0 + fbm_2d(x,z, 5) * 15 + ridge(x,z) * 5` |
| Swamp | 近海平面，零星水坑 | `f = y - 0.5 + simplex_2d(x,z) * 1.5 + spot(x,z) * 2` |

Biome 边界处对周围 3×3 采样加权混合。利用现有噪声函数 (`assets/shaders/noise/`) 替代注释代码。

## 验收

- 6 种 biome 地形形状肉眼可辨
- Biome 边界平滑，无断层
- `cargo run -p atom_terrain --example chunk_loader`

## 约束

- 修改 `assets/shaders/terrain/compute/` 下的 WGSL，不改 Rust compute 框架代码
- Biome 纹理通过现有 bind group 传入
- 不改变 17³ 计算网格
