# Atom Terrain Engine

基于 Bevy 的体素平滑地形系统，Dual Contouring + Probabilistic QEF 在 GPU 上生成网格，Voronoi 生物群系 + PBR 材质。

**Tech**: Rust Edition 2024, Bevy 0.18, wgpu 27.0, WGSL compute/render, encase/bytemuck

## 文档索引

| 位置 | 内容 |
|------|------|
| `README.md` | 用户安装与使用 |
| **项目宪法** | |
| `.claude/SOUL.md` | Agent 行为规范、依赖规则、测试策略、Spec 生命周期、架构边界 |
| `.claude/ARCHITECTURE.md` | 架构决策记录 (ADR) |
| `.claude/TENSIONS.md` | 摩擦日志（GPU/数据/工具链/流程/已知退化） |
| `.plan/` | 跨会话规划（PLAN + vision + MEMORY + DRIFT + SESSION-LOG） |
| **Agent 框架** | |
| `.claude/AGENTS.md` | 操作协议、缺口检测、跨会话 .plan/ 协议 |
| `.claude/CAPABILITY-MAP.md` | 当前能力边界（能做什么 / 缺什么 / 需人类判断） |
| `.claude/APPEND_SYSTEM.md` | 注入系统 prompt 的代码惯例 |
| **Skills** | |
| `.claude/skills/agent-spec-*` | Spec 工作流（authoring / estimate / tool-first） |
| `.claude/skills/obsidian-*` | Obsidian 集成（bases / cli / markdown） |
| `.claude/skills/defuddle/` | 网页内容提取 |
| `.claude/skills/json-canvas/` | JSON Canvas 编辑 |
| **工具** | |
| `Justfile` | 开发任务（`check`/`clippy`/`bevy-lint`/`test`/`deny`/`ci`） |
| `.githooks/` | pre-commit / pre-push 脚本 |
| `.claude/commands/` | 快捷命令（`/check` `/clippy` `/test` `/run`） |
| `.claude/rules/` | 规则定义（build-check、code-style） |
| `specs/*.spec` | 任务 Spec（Phase 边界、完成条件） |
### 命名与语言

- Rust 标准命名: `snake_case` 模块/函数, `CamelCase` 类型, UPPER_CASE 常量
- 非平凡逻辑用**中文**注释; 简单辅助函数用英文; Shader 中英文混合
- 模块组织: `mod.rs` 模式; 相关 component/system/resource 分组

### 错误处理

- 禁止 `unwrap()` (clippy: `unwrap_used = "warn"`), 统一使用 `expect("原因")`
- `panic!` 允许用于硬停止场景 (如找不到 `.atom.project`)
- 可恢复场景用 `warn!` 记录并跳过
- 不使用 `thiserror`/`anyhow` — 保持简单

### ECS 模式

```rust
// TerrainState 驱动阶段切换
#[derive(States)]
enum TerrainState { None, LoadAssets, GenerateTerrainRegion, GenerateTerrainMesh }

// SystemSet 链式执行
#[derive(SystemSet)]
enum TerrainSystems { ChunkLoader, ApplyCSG, GenerateChunk }

app.configure_sets(Update, (
    TerrainSystems::ChunkLoader,
    TerrainSystems::ApplyCSG,
    TerrainSystems::GenerateChunk,
).chain().run_if(in_state(TerrainState::GenerateTerrainMesh)));
```

### 渲染世界同步

```rust
// ExtractResource — Resource 同步到渲染世界
#[derive(Resource, Clone, ExtractResource)]
struct TerrainSetting { .. }
app.add_plugins(ExtractResourcePlugin::<TerrainSetting>::default());

// ExtractComponent — Component 同步到渲染世界
#[derive(Component, ExtractComponent)]
struct TerrainChunk;
app.add_plugins(ExtractComponentPlugin::<TerrainChunk>::default());

// crossbeam channel — 渲染→主世界 Mesh 数据传输
let (s, r) = crossbeam::channel::unbounded();
app.insert_resource(TerrainChunkMeshDataReceiver(r));
// render world 持有 Sender, main world 持有 Receiver
```

### 自定义 SystemParam

```rust
fn my_system(reader: TableReader<TbItem>, barrier: AllAssetBarrier) { .. }
```

### Shader 资源管理

通过 `shaders_plugin!` 宏自动生成 Shader 加载 Plugin 和 Resource:

```rust
// 通用形式 (自定义模块前缀)
shaders_plugin!(Terrain, Material, (
    triplanar_shader -> "shaders/terrain/planar/triplanar.wgsl",
    biplanar_shader -> "shaders/terrain/planar/biplanar.wgsl",
));
// 生成: TerrainMaterialShaders Resource + TerrainMaterialShadersPlugin Plugin
```

Shader 路径相对于 `assets/`。`file_watcher` 自动热重载。

### Material (标准 Bevy Material trait)

```rust
#[derive(AsBindGroup, Clone, Asset, TypePath)]
#[bind_group_data(TerrainMaterialKey)]
#[uniform(0, TerrainMaterialUniform)]
pub struct TerrainMaterial { .. }

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain/render/terrain_type.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef { ShaderRef::Default }
}

app.add_plugins(MaterialPlugin::<TerrainMaterial>::default());
```

### GPU Buffer 生命周期

```rust
use atom_render::shared_buffer::SharedStorageBuffer;

// 基础用法
let mut buf = SharedStorageBuffer::<MyType>::new(alignment);
buf.reserve(count);   // 分配
buf.write(&data);     // 写入
buf.push();           // 提交到 GPU
buf.clear();          // 重置

// 分级上传 (Staging)
use atom_render::staged_buffer::SharedStagedBuffer;
let mut staged = SharedStagedBuffer::<MyType>::new(alignment);
staged.write(&data);           // 写入 CPU staging buffer
staged.flush(render_queue);    // 提交到 GPU
let gpu_buf: &SharedStorageBuffer<MyType> = staged.get_gpu_buffer();
```

### LayerTag 分层标签

```rust
use atom_layertag::builder::LayerTagBuilder;

let tag = LayerTagBuilder::new()
    .add_tag(Tag::new("ability"))
    .add_tag(Tag::new("stun"))
    .add_tag(Tag::new("fire"))
    .build_single();

tag.exact_match(&other);      // 完全相等
tag.partial_match(&other);    // 前缀匹配
tag.same_prefix(&other);      // 同前缀迭代器

// 两种容器 (均为 Bevy Component)
SingleLayerTagContainer  // HashSet<LayerTag> — 去重语义
CountLayerTagContainer   // Vec<CountLayerTag> — 引用计数语义
```

LayerTag 需通过 `LayerTagRegistry` 注册。

### 数据表访问

```rust
// TableReader<T> 按 key 查询
fn read_table(reader: TableReader<TbItem>) {
    let item: Arc<Item> = reader.get_row("item_id")?;
}
```

加载流程: `DataTablePlugin` → `Wait` → `Loading` → `Loaded` → `TablesLoadedEvent`。
`AllAssetBarrier` 管理命名 barrier。

### TerrainSetting

```rust
#[derive(Resource, Clone, ExtractResource)]
pub struct TerrainSetting {
    pub chunk_setting: TerrainChunkSetting,  // { voxel_size, voxel_count }
    pub size_setting: TerrainSizeSetting,    // { height_range, horizontal_range }
    pub qef_solver: bool,
    pub qef_solver_threshold: f32,
    pub qef_stddev: f32,
}

// Getters
setting.get_voxel_count_in_chunk()    // -> u32
setting.get_voxel_count_in_compute()  // -> u32 (chunk 缝合边界多 1)
setting.get_chunk_size()              // -> f32 (世界单位)
setting.get_voxel_size()              // -> f32
setting.get_terrain_size()            // -> f32
setting.get_height_range_size()       // -> RangeInclusive<f32>
```

### GPU Mesh 管线 (4 Compute Pass)

每 chunk: `voxel_vertex_values` → `cross_points` → `vertices` → `indices`。
数据通过 crossbeam channel 从 render world 回传 main world。

### 项目根检测

`.atom.project` 标记文件，`ProjectPaths::root_path()` 从 CWD 向上遍历。

## 格式化

`rustfmt.toml`: Unix 换行, field init shorthand, edition 2024
`clippy.toml`: `unwrap_used = "warn"`, `too_many_arguments`/`type_complexity`/`collapsible_if` 允许
