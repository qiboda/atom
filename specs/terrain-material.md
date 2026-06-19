# Spec: Biome 驱动材质

> Phase 3 · 规划中

## 目标

根据 biome 类型为地形顶点赋予材质，不同 biome 有不同外观。

## 规则

每 biome 一个材质定义，渲染时根据顶点 biome 权重混合。

| Biome | 主色 | 粗糙度 | 金属度 |
|-------|------|--------|--------|
| Ocean | 深蓝灰 | 0.3 | 0.0 |
| Plains | 草绿 | 0.9 | 0.0 |
| Forest | 深绿 | 0.85 | 0.0 |
| Desert | 沙黄 | 0.95 | 0.0 |
| Mountains | 灰白 | 0.8 | 0.1 |
| Swamp | 暗绿棕 | 0.92 | 0.0 |

顶点权重混合: `albedo = Σ(w_i * mat_i.albedo)`，其中 w_i 是该顶点周边 8 个 voxel 角的 biome 类型比例。

## 验收

- 不同 biome 有不同颜色和 PBR 参数
- 边界处颜色平滑过渡
- 不增加 draw call

## 约束

- 只改 render shader，不改 compute pipeline
- 用已存在的 TerrainMaterial 扩展，不引入新材质系统
