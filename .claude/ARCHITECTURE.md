# Architecture Decision Records — Atom

> 记录关键架构决策及其理由。每条记录解决一个问题，说明背景、选项、决策和后果。

## 已做决策

### GPU Compute Mesh Generation
- **日期**: 2026-06
- **决策**: 4-pass compute shader pipeline (voxel vertices → cross points → QEF vertices → indices)
- **理由**: Dual Contouring 天然适合 GPU 并行，每个体素独立。Tretenner 2020 概率性 QEF 处理噪声法向不确定性
- **替代方案**: CPU Marching Cubes（太慢）、Compute + CPU readback between passes（GPU-CPU 同步开销）
- **后果**: 需管理复杂 buffer layout 和 dynamic offset；debug 困难（无 WGSL 断点）

### Biome 纹理作为密度场输入
- **日期**: 2026-06
- **决策**: CPU 生成 Voronoi 图 → Luma8 纹理 → GPU 双线性采样 biome 类型 (0-5)
- **理由**: 6 种 biome 足够区分；纹理查找比 21 种混合更简单；2×2 采样提供 biome 边界过渡
- **替代方案**: 全 GPU 21 种 biome height map (`height.wgsl`) — 复杂，暂未激活
- **后果**: Luma8 需 ×255 恢复 u8；biome 纹理 4096×4096 占用 16MB

### MaterialPlugin vs 自定义管线
- **日期**: 2026-06
- **决策**: `MaterialPlugin::<TerrainMaterial>` (AsBindGroup + specialize)
- **理由**: 复用 Bevy PBR 管线；只需自定义 vertex/fragment shader
- **后果**: BIOME_VERTEX_ATTRIBUTE @location(2) 需在 mesh 和 shader 间对齐；当前 TerrainMaterial 管线有渲染 bug，暂用 StandardMaterial

### 2D 高度图噪声替代 3D 噪声
- **日期**: 2026-06-19
- **决策**: `open_simplex_2d_fbm_with_seed(location.xz, ...)` 替代 `open_simplex_3d_with_seed`
- **理由**: 3D 噪声产生碎片化 isosurface（多值高度）；2D 保证唯一高度值 per (x,z)，产生连续地形
- **后果**: 失去悬挑/洞穴/拱门；换取平滑可预测的地表

### Shared Buffer + Dynamic Offset
- **日期**: 2026-06
- **决策**: 所有 chunk 共享 7 个 GPU buffer，dynamic offset 分离 per-chunk 数据
- **理由**: 减少 buffer 分配/Vulkan descriptor set 切换；multi-draw indirect 兼容
- **后果**: stride 必须统一（LOD 0 max 为标准）；staging buffer 读写需 offset 计算

---

## ADR 模板

```markdown
### [标题]
- **日期**: YYYY-MM-DD
- **状态**: proposed | accepted | deprecated | superseded
- **决策**: 我们决定做什么
- **背景**: 需要解决什么
- **选项**: 考虑过哪些方案
  - 方案 A (selected): 理由
  - 方案 B: 为什么不选
  - 方案 C: 为什么不选
- **后果**: 积极/消极影响
  - + 优势
  - - 代价/限制
- **关联**: 引用其他 ADR #[kebab-id]
```
