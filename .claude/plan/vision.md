# 程序化地形项目 — 方向

## 核心原则

算法驱动一切。地形、材质、生态——全部由系统生成。MVP 优先：先跑通 GPU 管线，再增加复杂度。

## 当前状态

- **基础**: Bevy 0.19 ECS，本地仓库 `/data/codes/Bevy`
- **Pipeline**: 四 pass GPU Dual Contouring，固定 slot 顶点/索引，CPU compact remap
- **地形**: 单 biome value noise 密度场（`y - height_at(x,z)`）
- **材质**: StandardMaterial 绿色，后续切换 TerrainMaterial
- **Workspace**: 仅 `atom_terrain` 一个 crate，其他后续逐步迁入

## 技术累积栈

1. **Rust MVP smoke test** (当前) — GPU 端到端验证
2. **多 chunk 动态加载** — Observer + chunk pool
3. **Biome 分布** (最后) — 6 biome 纹理驱动密度场
4. **材质** — biome 驱动 PBR (TerrainMaterial)
5. **寻路** — nav mesh from SDF
6. **CSG** — 洞穴/构造物系统生成
