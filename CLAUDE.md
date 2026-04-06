# CLAUDE.md — Atom Terrain Engine

## Project Overview

基于 Bevy 引擎的体素平滑地形系统，使用 **Dual Contouring + Probabilistic QEF** 在 GPU 上生成地形网格，支持生物群系驱动的地形生成和 CSG 地形编辑。目标是为游戏开发提供高质量的程序化地形。

## Tech Stack

- **Rust Edition 2024**, resolver v3
- **Bevy 0.18** (features: `file_watcher`, `embedded_watcher`, `trace`, `trace_tracy`, `serialize`, `bevy_remote`)
- **wgpu 27.0** / **encase 0.12** / **bytemuck 1.24**
- **WGSL** compute & render shaders
- Dev tools: `bevy-inspector-egui 0.36`, `bevy_flycam 0.18`

## Workspace Crates

| Crate | Purpose |
|-------|---------|
| `atom_terrain` | **核心**: 地形系统（chunk管理、GPU mesh生成、biome、材质） |
| `atom_pqef` | Probabilistic Quadric Error Function 求解器 |
| `atom_render` | GPU buffer 抽象 (SharedStorageBuffer, SharedUniformBuffer, SharedStagedBuffer) |
| `atom_shader_lib` | Shader 资源加载插件 + 调试形状渲染 |
| `atom_math` | 三角形数学工具（重心坐标、光栅化） |
| `atom_core` | 项目级工具（日志、路径解析） |
| `atom_renderdoc` | RenderDoc 集成 (F12 启动回放) |

## Architecture

### Terrain State Machine

`TerrainState`: `None` → `LoadAssets` → `GenerateTerrainRegion` → `GenerateTerrainMesh`

在 `GenerateTerrainMesh` 状态下，三个 SystemSet 按顺序执行：
1. `ChunkLoader` — 根据 `TerrainObserver` 位置加载/卸载 chunk
2. `ApplyCSG` — CSG 地形修改（规划中）
3. `GenerateChunk` — 触发 GPU compute mesh 生成

### GPU Mesh Generation Pipeline (4 Compute Passes)

1. **compute_voxel_vertices**: 在 (N+1)³ 网格上计算密度场值
2. **compute_voxel_cross_points**: 二分搜索（8次迭代）找边与等值面交点，中心差分求法线
3. **compute_vertices**: 收集每个 voxel 的 12 条边交叉数据，PQEF 最小化求顶点位置
4. **compute_indices**: 检测符号变化的边，每条边生成 2 个三角形（6 个索引）

数据通过 crossbeam channel 从 render world 传回 main world。

### Terrain Settings (defaults)

- Voxel size: 0.5m
- Voxels per chunk per axis: 16 (chunk = 8m)
- Compute grid: 17³ (额外一层处理 chunk 边界缝合)
- Height range: -8 to +16 chunks
- Max terrain: 4096m × 4096m

### Biome System

使用 Voronoi 图生成 2D 生物群系区域：
- 类型: Ocean, Forest, Desert, Plains, Mountains, Swamp
- 从种子点扩展，产生不规则形状
- 光栅化为灰度图像用于 GPU 采样

### Custom Material

`TerrainMaterial` 支持三面/双面纹理投影，双 biome 纹理集，自定义顶点属性 (`biome`)。

## Key File Paths

```
crates/atom_terrain/src/terrain/mod.rs      — 地形插件 + 状态机
crates/atom_terrain/src/terrain/setting.rs   — TerrainSetting 配置
crates/atom_terrain/src/chunks/loader/       — chunk 加载器
crates/atom_terrain/src/chunks/mesh/         — mesh 生成系统
crates/atom_terrain/src/chunks/mesh/compute/ — GPU compute pipeline
crates/atom_terrain/src/chunks/mesh/materials/ — 自定义材质
crates/atom_terrain/src/biomes/              — 生物群系系统
crates/atom_pqef/src/quadric.rs              — QEF 求解器
assets/shaders/terrain/compute/              — compute shaders
assets/shaders/terrain/render/               — render shaders
assets/shaders/noise/                        — 噪声函数
assets/shaders/quadric/                      — GPU 端 QEF
```

## Build & Run

```bash
# Run the chunk_loader example (主要测试入口)
cargo run -p atom_terrain --example chunk_loader

# Check compilation
cargo check --workspace

# Clippy
cargo clippy --workspace
```

Build profiles:
- `dev`: opt-level=1, dependencies opt-level=3
- `release`: codegen-units=1, LTO=thin
- `wgpu-types`: debug-assertions disabled (Bevy #14291)

## Code Conventions

- **命名**: Rust 标准命名，结构体/函数用英文
- **注释**: 主要使用**中文**，shader 中英文混合
- **Clippy**: 允许 `too_many_arguments`, `type_complexity`（Bevy 系统签名常见）；`unwrap_used` 警告，优先使用 `expect`
- **格式化**: `rustfmt.toml` 配置了 field init shorthand 和 Unix 换行
- **模块组织**: 使用 `mod.rs` 模式，相关的 component/system/resource 分组在子模块中
- **GPU 数据回传**: 通过 crossbeam channel 从 render world 发送到 main world
- **项目根检测**: 使用 `.atom.project` 标记文件

## Development Roadmap

1. ~~实现生态系统（biome）~~ ✅ 基础 Voronoi 区域生成已完成
2. 🔨 根据生态系统生成地形形状（密度场集成 biome 数据 — 进行中）
3. 📋 基于地形和生态系统添加材质
4. 📋 实现 CSG 支持地形修改

## Important Notes

- Shader 修改后会通过 `file_watcher` 自动热重载
- 密度场当前为简单平面 (`y - 5.0`)，噪声和高度图代码已注释待集成
- Render world 和 Main world 之间通过 channel 异步通信，注意线程安全
- QEF 求解在 GPU 和 CPU 都有实现（`atom_pqef` + `quadric.wgsl`）
