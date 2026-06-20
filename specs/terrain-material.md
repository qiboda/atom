# Spec: Biome 驱动材质

> Phase 3 · 规划中
> 依赖: Phase 2 (biome 密度场集成完成)


## 目标

根据 biome 类型为顶点赋予不同材质参数，不同 biome 有不同外观。利用已有的 `BIOME_VERTEX_ATTRIBUTE` 在 render shader 中根据 biome 混合。

## 规则

每 biome 一组 PBR 参数，渲染时根据顶点 biome_vertex_attribute 选择/混合。

| Biome | 主色 | 粗糙度 | 金属度 |
|-------|------|--------|--------|
| Ocean | 深蓝灰 | 0.3 | 0.0 |
| Plains | 草绿 | 0.9 | 0.0 |
| Forest | 深绿 | 0.85 | 0.0 |
| Desert | 沙黄 | 0.95 | 0.0 |
| Mountains | 灰白 | 0.8 | 0.1 |
| Swamp | 暗绿棕 | 0.92 | 0.0 |

在 `terrain_type.wgsl` render shader 中读取 `biome` vertex attribute，设置对应的 roughness/metallic/albedo。

## 验收

- 不同 biome 有不同颜色和 PBR 参数
- 利用已存在的 `BIOME_VERTEX_ATTRIBUTE` 传递 biome 信息

## 约束

- 只改 render shader + TerrainMaterial::specialize()，不改 compute pipeline
- 复用已存在的 `MaterialPlugin::<TerrainMaterial>` 和 planar shaders
- `TerrainMaterialUniform` 的 roughness/metallic 字段可复用
