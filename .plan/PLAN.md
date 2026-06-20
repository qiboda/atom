# PLAN — 程序化地形管线

## Phase 1 — Biome 生态 ✅

- [x] 1.1 Voronoi 区域生成
- [x] 1.2 6 种 biome 定义
- [x] 1.3 GPU 可采样纹理

## Phase 2 — 地形形状 ✅

> Spec: `specs/terrain-shape.spec`

- [x] 2.1 6 种 biome 密度场函数（GPU shader）— `density_field.wgsl`
- [x] 2.2 Biome 边界加权混合 — 2×2 双线性
- [x] 2.3 激活现有噪声函数 — `open_simplex_3d_with_seed`
- [x] 2.4 验证：chunk(-2,0,-2) 1493 顶点 / 8214 索引 (2738 三角形)

Exit: `cargo run -p atom_terrain --example chunk_loader` 生成地形网格。

## Phase 3 — 材质 ✅

> Spec: `specs/terrain-material.spec`

- [x] 3.1 6 种 biome PBR 参数定义 — `BIOME_COLORS` + `BiomeColorUniform`
- [x] 3.2 顶点 biome 属性传递 — `get_vertex_biome()` + `BIOME_VERTEX_ATTRIBUTE`
- [x] 3.3 WGSL TerrainMaterial 扩展 — `biome_colors: array<BiomeColor, 6>`
- [x] 3.4 构建通过 + 网格正常生成

Exit: 不同 biome 顶点有不同 base_color；颜色混合通过 biome_weights 计算。

## Phase 4 — LOD 🔨

> Spec: `specs/lod.spec`

- [x] 4.1 TerrainChunkLod 组件 + from_distance + update_chunk_lod 系统
- [x] 4.2 LOD 变更重新触发 meshing
- [ ] 4.3 可变 compute 分辨率 (per-chunk voxel_num) — 需 buffer 重构
- [ ] 4.4 256m 可视距离验证

Exit (当前): LOD 架构就绪，变更检测 + 重触发正常。实际分辨率切换留待 Phase 4.1。

---

## 后续储备

不在当前 plan 中，方向定义在 `.plan/vision.md`：

- 寻路 (nav mesh)
- 技能系统与地形交互
- 怪物 AI 生成规则
- CSG 洞穴/构造物生成（系统驱动，非玩家编辑）
- 地形存储与加载
