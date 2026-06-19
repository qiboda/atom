# Spec: 基于 Biome 的地形形状

> Phase 2 · 进行中

## 目标

将 biome 数据集成到密度场，使 6 种 biome 产生不同的地形形状。不再使用全局统一的 `y - 5.0`。

## 规则

每个 biome 定义自己的密度场函数，GPU compute shader 中根据 biome 采样选择。

| Biome | 密度场特征 | 实现 |
|-------|----------|------|
| Ocean | 低于海平面，平坦底面 | `f = y + 8.0` |
| Plains | 低起伏，开阔 | `f = y - 2.0 + simplex_2d(x,z) * 2` |
| Forest | 中等起伏，有自然坡度 | `f = y - 3.0 + fbm_2d(x,z, 3) * 4` |
| Desert | 平坦，有沙丘波纹 | `f = y - 1.0 + |simplex_2d(x,z)| * 3` |
| Mountains | 剧烈起伏，尖锐山峰 | `f = y - 10.0 + fbm_2d(x,z, 5) * 15 + ridge(x,z) * 5` |
| Swamp | 近海平面，零星水坑 | `f = y - 0.5 + simplex_2d(x,z) * 1.5 + spot(x,z) * 2` |

Biome 边界处，对周围 3×3 采样点的密度场值按 biome 比例加权混合。

## 验收

- 6 种 biome 地形形状肉眼可辨
- Biome 边界平滑，无断层
- `cargo run -p atom_terrain --example chunk_loader`

## 约束

- 计算在现有 GPU compute pass 1 中完成
- biome 纹理通过 bind group 传入
- 不改变 (N+1)³ 计算网格
