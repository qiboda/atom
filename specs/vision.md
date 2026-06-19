# 程序化游戏项目 — 方向

## 核心原则

算法驱动一切。地形、生态、材质、遭遇——全部由系统生成。

## 当前状态

- **基础已就绪**: 13 个 crate workspace，Bevy 0.18 ECS
- **Phase 1 完成**: Voronoi biome 区域生成 + 光栅化纹理
- **Phase 2 进行中**: Biome → 密度场 (compute shader 框架已就绪, 密度场函数待接入)
- **Render pipeline 已就绪**: 4-pass GPU compute → crossbeam channel → render world
- **Material 框架已就绪**: TerrainMaterial (标准 Material trait), planar shaders, BIOME_VERTEX_ATTRIBUTE
- **atom_ability**: EffectGraph 技能系统已有完整框架

## 技术累积栈

1. **地形形状** (进行中) — biome 驱动密度场
2. **材质** — biome 驱动 PBR 参数
3. **LOD** — 远距离替代几何
4. **寻路** — nav mesh
5. **怪物 AI** — biome 生成规则
6. **CSG** — 洞穴/构造物系统生成
