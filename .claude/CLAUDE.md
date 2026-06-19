# Atom Terrain Engine

基于 Bevy 引擎的体素平滑地形系统，使用 **Dual Contouring + Probabilistic QEF** 在 GPU 上生成地形网格，支持生物群系驱动地形生成和 CSG 地形编辑。同时包含基于 **LayerTag** 分层的技能系统 (`atom_ability`)，使用 Luban 数据表驱动配置。

## Tech Stack

- **Rust Edition 2024**, resolver v3
- **Bevy 0.18** (features: `file_watcher`, `embedded_watcher`, `trace`, `trace_tracy`, `serialize`, `bevy_remote`)
- **wgpu 27.0** / encase 0.12 / bytemuck 1.24
- **WGSL** compute & render shaders (~75+ files)
- Dev tools: `bevy-inspector-egui 0.36`, `bevy_flycam 0.18`

## Architecture

```
┌────────────────────────────────────────────────────┐
│                  atom_terrain (核心)                 │
│  地形状态机 → Chunk管理 → GPU Compute → 材质渲染     │
├────────────────────────────────────────────────────┤
│ atom_ability     atom_cel_shader  atom_shader_lib   │
│ (技能系统)        (赛璐璐渲染)      (Shader加载+调试) │
├────────────────────────────────────────────────────┤
│ atom_render      atom_pqef     atom_renderdoc       │
│ (GPU Buffer抽象) (QEF求解器)   (RenderDoc集成)       │
├────────────────────────────────────────────────────┤
│ atom_datatables   atom_layertag  atom_math          │
│ (Luban数据表)     (分层标签)     (三角形数学)         │
├────────────────────────────────────────────────────┤
│ atom_utils        atom_core                         │
│ (AssetBarrier)    (日志/路径)                        │
└────────────────────────────────────────────────────┘
```

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

### Data Flows

- **地形生成**: CPU Voronoi 图 → biome 纹理 → GPU Compute (4 pass) → biplanar/triplanar PBR 渲染
- **Render ↔ Main 通信**: crossbeam channel 异步传递 mesh 数据
- **配置数据**: Excel (Luban .conf) → Luban .NET 代码生成 → `atom_cfg` Rust + `.bytes` → Bevy AssetLoader → `TableReader<T>` SystemParam

### ECS 模式

- **状态机**: `TerrainState` 驱动阶段切换
- **SystemSet 链**: `ChunkLoader → ApplyCSG → GenerateChunk` 按序执行
- **依赖注入**: Bevy `Resource`/`Component` + 自定义 `SystemParam`（如 `TableReader<T>`）
- **提取模式**: 渲染世界数据标记 `ExtractResource` / `ExtractComponent`
- **GPU 数据回传**: crossbeam channel 从 render world 发送到 main world

### Biomes

使用 Voronoi 图生成 2D 生物群系区域：Ocean, Forest, Desert, Plains, Mountains, Swamp。从种子点扩展产生不规则形状，光栅化为灰度图像供 GPU 采样。

### Terrain Settings (defaults)

- Voxel size: 0.5m, 16 voxels per chunk per axis (chunk = 8m)
- Compute grid: 17³ (额外一层处理 chunk 边界缝合)
- Height range: -8 to +16 chunks, Max terrain: 4096m × 4096m
- Custom material: `TerrainMaterial` 支持 biplanar/triplanar 纹理投影，双 biome 纹理集

### Project Root Detection

使用 `.atom.project` 标记文件，`ProjectPaths::root_path()` 从 CWD 向上遍历查找。

## Workspace Crates

| Crate | Purpose |
|-------|---------|
| `atom_terrain` | **核心**: 地形系统（chunk管理、GPU mesh生成、biome、材质） |
| `atom_pqef` | Probabilistic QEF 求解器 (Trettner 2020, CPU+GPU 双实现) |
| `atom_render` | GPU buffer 抽象 (SharedStorageBuffer, SharedUniformBuffer, StagedBuffer) |
| `atom_shader_lib` | Shader 资源加载宏 + 调试形状渲染 (三角形/线/点) |
| `atom_cel_shader` | 赛璐璐材质 + 背面描边 (CelMaterial, BackFacingMaterial) |
| `atom_ability` | 技能系统 (LayerTag 条件 + EffectGraph 流程图 + 数据表驱动) |
| `atom_datatables` | Luban 数据表系统 (状态机加载、TableReader SystemParam) |
| `atom_layertag` | 分层标签系统 (点分隔路径、精确/部分/前缀匹配) |
| `atom_math` | 三角形数学工具（重心坐标、光栅化） |
| `atom_core` | 项目基础设施（日志初始化、路径解析） |
| `atom_utils` | `AssetBarrier` 自定义异步屏障 |
| `atom_renderdoc` | RenderDoc 集成 (F12 启动回放) |
| `atom_datatables/gen/` | Luban 生成代码 (`atom_cfg`, `atom_macros`) |
| `atom_datatables/atom_luban_lib/` | ByteBuf 二进制反序列化器 |

## Key File Paths

```
crates/atom_terrain/src/terrain/mod.rs      — 地形插件 + 状态机
crates/atom_terrain/src/terrain/setting.rs   — TerrainSetting 配置
crates/atom_terrain/src/chunks/loader/       — chunk 加载器
crates/atom_terrain/src/chunks/mesh/compute/ — GPU compute pipeline
crates/atom_terrain/src/chunks/mesh/materials/ — 自定义材质
crates/atom_terrain/src/biomes/generator.rs   — Voronoi 生物群系生成器
crates/atom_pqef/src/quadric.rs              — QEF 求解器
crates/atom_datatables/src/tables_system_param.rs — TableReader SystemParam
crates/atom_layertag/src/layertag.rs          — LayerTag 核心定义
crates/atom_core/src/paths.rs                 — 项目路径解析
crates/atom_core/src/logger.rs                — 日志初始化
assets/shaders/terrain/compute/              — GPU compute shaders (4 pass)
assets/shaders/terrain/render/               — 地形渲染 shaders
assets/shaders/noise/                        — 噪声函数 (OpenSimplex, FBM, Perlin 等)
assets/shaders/quadric/                      — GPU 端 QEF
assets/shaders/csg/                          — SDF 基元 (30+ shapes from IQ)
Cargo.toml                                  — Workspace 定义、依赖、clippy lints、profiles
rustfmt.toml                                — Unix 换行、field init shorthand
clippy.toml                                 — doc-valid-idents 白名单
atom.code-workspace                          — VS Code 工作区配置
```

## Build & Run

```bash
# 主要测试入口 (地形 chunk 加载)
cargo run -p atom_terrain --example chunk_loader

# 其他示例
cargo run -p atom_ability --example ability        # 技能系统
cargo run -p atom_shader_lib --example show_noise  # 噪声着色器
cargo run -p atom_cel_shader --example back_facing # 背面描边
cargo run -p atom_datatables --example tables_load # 数据表加载

# 检查/格式化
cargo check --workspace
cargo clippy --workspace

# 数据表代码生成 (需要 .NET Luban 工具链)
crates/atom_datatables/gen_bin.bat
```

### Build Profiles

- **dev**: `opt-level=1` (自身) / `opt-level=3` (依赖); wgpu-types `debug-assertions=false` (Bevy #14291)
- **release**: `codegen-units=1`, `lto="thin"`

## Key Dependencies

| 依赖 | 版本 | 用途 |
|------|------|------|
| bevy | 0.18 | 游戏引擎 (ECS + 渲染 + 资产) |
| wgpu | 27.0 | GPU API 抽象 |
| encase | 0.12 | GPU shader 类型编码 |
| bytemuck | 1.24 | 零开销字节转换 |
| crossbeam | 0.8 | 跨线程 channel (渲染世界→主世界) |
| voronator | — | Centroidal Voronoi 图生成 |
| tracing | 0.1 | 结构化日志 |
| once_cell | 1.21 | 惰性初始化 |
| smallvec | 1.15 | 小数组优化 |
| thiserror | 2 | (workspace 依赖, 暂未使用) |

## Development Roadmap

1. ~~实现生态系统（biome）~~ ✅ 基础 Voronoi 区域生成已完成
2. 🔨 根据生态系统生成地形形状（密度场集成 biome 数据 — 进行中）
3. 📋 基于地形和生态系统添加材质
4. 📋 实现 CSG 支持地形修改

## Important Notes

- Shader 修改后通过 `file_watcher` 自动热重载
- 密度场当前为简单平面 (`y - 5.0`)，噪声和高度图代码已注释待集成
- Render world 和 Main world 之间通过 channel 异步通信，注意线程安全
- QEF 求解在 GPU 和 CPU 都有实现（`atom_pqef` + `quadric.wgsl`）
- 文档位于 `doc/`，Obsidian 格式，按四元素分类（技术/故事/机制/美学）
- Python 工具链用 `uv` 管理（`tools/pythons/`），仅用于 pre-commit
