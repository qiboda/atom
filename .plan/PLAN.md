# PLAN — 程序化地形管线

## Phase 1 — Biome 生态 ✅

- [x] 1.1 Voronoi 区域生成
- [x] 1.2 6 种 biome 定义
- [x] 1.3 GPU 可采样纹理

## Phase 2 — 地形形状 🔨

> Spec: `specs/terrain-shape.md`

- [ ] 2.1 6 种 biome 密度场函数（GPU shader）
- [ ] 2.2 Biome 边界加权混合
- [ ] 2.3 激活现有噪声函数
- [ ] 2.4 验证：6 种地形肉眼可辨

Exit: `cargo run -p atom_terrain --example chunk_loader` 中不同 biome 有不同高度和起伏。

## Phase 3 — 材质 📋

> Spec: `specs/terrain-material.md`

- [ ] 3.1 6 种 biome PBR 参数定义
- [ ] 3.2 顶点权重材质混合
- [ ] 3.3 TerrainMaterial 扩展
- [ ] 3.4 验证：biome 颜色/材质可辨

Exit: 不同 biome 有不同外观，边界平滑。

## Phase 4 — LOD 📋

> Spec: `specs/lod.md`

- [ ] 4.1 3 级 LOD 定义 (base/2×/4×)
- [ ] 4.2 接缝处理
- [ ] 4.3 验证：256m 可视距离，接缝不可见

Exit: 远距离 Chunk 三角形数明显降低。

---

## 后续储备

不在当前 plan 中，方向定义在 `specs/vision.md`：

- 寻路 (nav mesh)
- 技能系统与地形交互
- 怪物 AI 生成规则
- CSG 洞穴/构造物生成（系统驱动，非玩家编辑）
- 地形存储与加载
