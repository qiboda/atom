# Atom Terrain Engine

基于 Bevy 引擎的体素平滑地形系统，使用 **Dual Contouring + Probabilistic QEF** 在 GPU 上生成地形网格，支持生物群系驱动地形生成和 CSG 地形编辑。同时包含基于 **EffectGraph** 节点图的技能系统 (`atom_ability`)，使用 Luban 数据表驱动配置。

## Tech Stack

- **Rust Edition 2024**, resolver v3
- **Bevy 0.18** (features: `file_watcher`, `embedded_watcher`, `trace`, `trace_tracy`, `serialize`, `bevy_remote`)
- **wgpu 27.0** / wgpu-types 27.0 / encase 0.12 / bytemuck 1.24
- **WGSL** compute & render shaders
- Dev tools: `bevy-inspector-egui 0.36`, `bevy_flycam 0.18`

## Architecture

```
┌────────────────────────────────────────────────────┐
│                  atom_terrain (核心)                 │
│  地形状态机 → Chunk管理 → GPU Compute → 材质渲染     │
├────────────────────────────────────────────────────┤
│ atom_ability     atom_cel_shader  atom_shader_lib   │
│ (EffectGraph技能) (赛璐璐渲染)     (Shader加载+调试)  │
├────────────────────────────────────────────────────┤
│ atom_render      atom_pqef     atom_renderdoc       │
│ (GPU Buffer抽象)  (QEF求解器)   (RenderDoc集成)       │
├────────────────────────────────────────────────────┤
│ atom_datatables   atom_layertag  atom_math          │
│ (Luban数据表)     (分层标签)      (三角形数学)         │
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

`TerrainChunkMeshComputeNode` 实现 `render_graph::Node`，每 chunk 执行 4 个 compute pass：

1. **compute_voxel_vertex_values**: 在 (N+1)³ 网格上计算密度场值（N=voxel_count=16，即 17³ grid）
2. **compute_voxel_cross_points**: 二分搜索（8次迭代）找边与等值面交点
3. **compute_vertices**: QEF 最小化求顶点位置
4. **compute_indices**: 检测符号变化的边，生成三角形索引

数据通过 `crossbeam::channel::unbounded` 从 render world 传回 main world（`TerrainChunkMeshDataSender` render 端 + `TerrainChunkMeshDataReceiver` main 端）。

### Data Flows

- **地形生成**: CPU Voronoi 图 → biome 纹理(灰度图) → GPU Compute (4 pass) → triplanar/biplanar PBR 渲染
- **Render ↔ Main 通信**: crossbeam channel 异步传递 mesh 数据
- **配置数据**: Excel (Luban .conf) → Luban .NET 代码生成 → `atom_cfg` Rust + `.bytes` → Bevy AssetLoader → `TableReader<T>` SystemParam

### ECS 模式

- **状态机**: `TerrainState` 驱动阶段切换
- **SystemSet 链**: `ChunkLoader → ApplyCSG → GenerateChunk` 在 `in_state(TerrainState::GenerateTerrainMesh)` 时运行
- **依赖注入**: Bevy `Resource`/`Component` + 自定义 `SystemParam`（如 `TableReader<T>`）
- **提取模式**: 渲染世界数据标记 `ExtractResource` / `ExtractComponent`
- **GPU 数据回传**: crossbeam channel 从 render world 发送到 main world

### Biomes

使用 Voronoi 图 + 质心抖动 + 区域扩展生成 2D 生物群系区域。

`BiomeType` enum: `Ocean=0, Forest=1, Desert=2, Plains=3, Mountains=4, Swamp=5`

`TerrainRegionGeneratorSetting` 默认值:
- `grid_cell_size`: 32.0
- `point_jitter_range`: 0.2..0.8
- `area_range`: 0.2..0.8
- `rand_area_setting`: Forest(2-4 areas), Desert(1), Plains(2-4), Mountains(1-2), Swamp(1-2)

Pipeline: `generate_centroid_diagram` → `generate_area_data` → `generate_biome_image` → `to_generate_terrain_mesh`

输出: `biome_image` (`Handle<Image>`), 灰度图，每个像素 = biome type ID。

### Terrain Settings

```rust
TerrainSetting {
    chunk_setting: TerrainChunkSetting { voxel_size: 0.5, voxel_count: 16 },  // chunk = 8m
    size_setting: TerrainSizeSetting { height_range: -8..=16, horizontal_range: 0..=511 },
    qef_solver: true, qef_solver_threshold: 0.1, qef_stddev: 0.1,
}
```

Helper methods: `get_voxel_count_in_chunk()`, `get_voxel_count_in_compute()` (N+1), `get_chunk_size()`, `get_height_range_size()`, `get_terrain_size()`, `is_in_height_range()`

### Material System

`TerrainMaterial` 实现标准 Bevy `Material` trait（不是 CustomMaterial）。`TerrainMaterialPlugin` 使用 `MaterialPlugin::<TerrainMaterial>`。

- `TerrainMaterialShader { triplanar, biplanar, terrain_material }` handles
- `BIOME_VERTEX_ATTRIBUTE`: `MeshVertexAttribute::new("biome", 100, VertexFormat::Uint32)`
- `TerrainMaterialUniform`: `lod: u32, roughness: f32, metallic: f32, flags: u32, reflectance: f32, attenuation_distance: f32, attenuation_color: Vec4`
- 支持双 biome 纹理集 (biome 0 + biome 1)，triplanar/biplanar 投影

### Isosurface

`atom_terrain/src/isosurface/mod.rs`:
```rust
pub enum IsosurfaceSide { Inside, Outside }
impl From<bool> for IsosurfaceSide  // true → Outside, false → Inside
```

### Project Root Detection

使用 `.atom.project` 标记文件，`ProjectPaths::root_path()` 从 CWD 向上遍历查找。

## Workspace Crates

| Crate | Purpose |
|-------|---------|
| `atom_terrain` | **核心**: 地形系统（chunk管理、GPU mesh生成、biome、Material、isosurface） |
| `atom_pqef` | Probabilistic QEF 求解器 (Trettner 2020, CPU+GPU 双实现: `math.rs`+`quadric.rs` + `quadric.wgsl`+`math.wgsl`) |
| `atom_render` | GPU buffer 抽象 (`SharedStorageBuffer<T>`, `StagedBuffer<T>`) |
| `atom_shader_lib` | `shaders_plugin!` 宏加载 Shader + 调试形状渲染 (`shapes/`) + GPU 数值测试 (`gpu_test/`) |
| `atom_cel_shader` | 赛璐璐材质 + 背面描边 (`CelMaterial`, `BackFacingMaterial` — 标准 Material trait) |
| `atom_ability` | EffectGraph 技能系统 (`graph/`, `effect/`, `buff/`, `ability/`, `attribute/`, `stateset/`)，含 `StateLayerTagRegistry`, `StateSet`, 节点图+黑板+上下文 |
| `atom_datatables` | Luban 数据表系统 (状态机 `Wait→Loading→Loaded`, `AllAssetBarrier`+`AsyncComputeTaskPool` 异步加载, `TableReader` SystemParam) |
| `atom_layertag` | 分层标签系统 (`LayerTag` 点分隔路径, `exact_match()`/`partial_match()`/`same_prefix()`, `LayerTagBuilder`, `LayerTagRegistry`) |
| `atom_math` | 三角形数学工具（重心坐标、光栅化） |
| `atom_core` | 项目基础设施 (`ProjectPaths::root_path()/saved_path()/assets_path()/config_root_path()/processed_assets_path()`, `logger.rs` tracing 初始化) |
| `atom_utils` | `AssetBarrier` (`Arc<AtomicUsize> + Mutex<Option<Waker>>`, 自定义 Future), `AllAssetBarrier` |
| `atom_renderdoc` | RenderDoc 集成 (F12 启动回放) |
| `atom_cfg` | Luban 生成的 Rust 配置代码 (`crates/atom_datatables/gen/atom_cfg/`) |
| `atom_macros` | Luban 生成的宏 (`crates/atom_datatables/gen/atom_macros/`) |
| `atom_luban_lib` | ByteBuf 二进制反序列化器 (`crates/atom_datatables/atom_luban_lib/`) |

## Key File Paths

```
crates/atom_terrain/src/terrain/mod.rs          — 地形插件 + 状态机
crates/atom_terrain/src/terrain/setting.rs       — TerrainSetting 配置
crates/atom_terrain/src/chunks/loader/            — chunk 加载器
crates/atom_terrain/src/chunks/mesh/compute/      — GPU compute pipeline (4 pass)
crates/atom_terrain/src/chunks/mesh/materials/    — TerrainMaterial
crates/atom_terrain/src/biomes/generator.rs       — Voronoi 生物群系生成器
crates/atom_terrain/src/isosurface/mod.rs         — IsosurfaceSide 枚举
crates/atom_pqef/src/quadric.rs                   — QEF 求解器 (CPU)
crates/atom_datatables/src/tables_system_param.rs — TableReader SystemParam
crates/atom_layertag/src/layertag.rs              — LayerTag 核心定义
crates/atom_shader_lib/src/shader_plugin_macro.rs — shaders_plugin! 宏
crates/atom_core/src/paths.rs                     — 项目路径解析
crates/atom_core/src/logger.rs                    — 日志初始化
assets/shaders/terrain/compute/                   — GPU compute shaders (4 pass)
assets/shaders/terrain/render/                    — 地形渲染 shaders
assets/shaders/noise/                             — 噪声函数 (OpenSimplex, FBM, Perlin 等)
assets/shaders/quadric/                           — GPU 端 QEF
assets/shaders/csg/                               — SDF 基元 (30+ shapes from IQ)
Cargo.toml                                       — Workspace 定义、依赖、clippy lints、profiles
rustfmt.toml                                     — Unix 换行、field init shorthand
clippy.toml                                      — doc-valid-idents 白名单
atom.code-workspace                              — VS Code 工作区配置
.claude/TENSIONS.md                               — 摩擦日志（GPU/数据/工具链/流程）
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
| wgpu-types | 27.0 | GPU 类型定义 |
| encase | 0.12 | GPU shader 类型编码 |
| bytemuck | 1.24 | 零开销字节转换 |
| crossbeam | 0.8 | 跨线程 channel (渲染世界→主世界) |
| rand | 0.9.2 | 随机数 (生物群系生成) |
| voronator | 0.2.1 | Centroidal Voronoi 图生成 (atom_terrain) |
| tracing | 0.1 | 结构化日志 |
| tracing-subscriber | 0.3 | tracing 订阅器 (env-filter, fmt) |
| tracing-appender | 0.2 | 日志文件追加 |
| once_cell | 1.21 | 惰性初始化 |
| serde | 1.0 | 序列化/反序列化 |
| bitflags | 2.10 | 位标志 (含 serde feature) |
| smallvec | 1.15 | 小数组优化 |
| thiserror | 2 | (workspace 依赖, atom_ability 使用) |

## Development Roadmap

1. ~~实现生态系统（biome）~~ ✅ 基础 Voronoi 区域生成已完成
2. 🔨 根据生态系统生成地形形状（密度场集成 biome 数据 — 进行中）
3. 📋 基于地形和生态系统添加材质
4. 📋 实现 CSG 支持地形修改

## Important Notes

- Shader 修改后通过 `file_watcher` 自动热重载
- 密度场当前为简单平面 (`y - 5.0`)，噪声和高度图代码已注释待集成
- Render world 和 Main world 之间通过 crossbeam channel 异步通信，注意线程安全
- QEF 求解在 GPU 和 CPU 都有实现（`atom_pqef` + `quadric.wgsl`）
- `LayerTag::new()` 接受 `Vec<Tag>`，使用 `LayerTagBuilder` 构建
- 文档位于 `doc/`，Obsidian 格式，按四元素分类（技术/故事/机制/美学）
