# Project-Specific Rules

## 代码约定

### 命名与语言

- Rust 标准命名: `snake_case` 模块/函数, `CamelCase` 类型, UPPER_CASE 常量
- 非平凡逻辑用**中文**注释; 简单辅助函数用英文; Shader 中英文混合
- 模块组织: `mod.rs` 模式; 相关 component/system/resource 分组建子模块

### 错误处理

- 禁止 `unwrap()` (clippy: `unwrap_used = "warn"`), 统一使用 `expect("原因")`
- `panic!` 允许用于硬停止场景 (如找不到 `.atom.project`)
- 可恢复场景用 `warn!` 记录并跳过
- 不使用 `thiserror`/`anyhow` — 保持简单

### Clippy

- 允许: `too_many_arguments`, `type_complexity` (Bevy ECS 系统签名的自然结果), `collapsible_if`
- 警告: `unwrap_used`

### 格式化

`rustfmt.toml`: Unix 换行, field init shorthand, edition 2024

## 通用模式

### ECS 状态与阶段驱动

```rust
// TerrainState 驱动阶段转换
#[derive(States)]
enum TerrainState { None, LoadAssets, GenerateTerrainRegion, GenerateTerrainMesh }

// TerrainSystems 链式执行
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
shaders_plugin!(
    Terrain, Material,
    (
        triplanar_shader -> "shaders/terrain/planar/triplanar.wgsl",
        biplanar_shader -> "shaders/terrain/planar/biplanar.wgsl",
    )
);
// 生成: TerrainMaterialShaders Resource + TerrainMaterialShadersPlugin Plugin

```

Shader 路径相对于 `assets/`。Shader 修改后 `file_watcher` 自动热重载，不需要重启。

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
    // specialize, alpha_mode, ...
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

tag.exact_match(&other);      // 完全相等 (a.b.c == a.b.c)
tag.partial_match(&other);    // 前缀匹配 (a.b 匹配 a.b.c, a.b.c.d 等)
tag.same_prefix(&other);      // 同前缀迭代器

// 两种容器 (均为 Bevy Component)
SingleLayerTagContainer  // HashSet<LayerTag> — 去重语义
CountLayerTagContainer   // Vec<CountLayerTag> — 引用计数语义
```

LayerTag 需先通过 `LayerTagRegistry` 注册才能使用。配置表 `TbLayerTag` 提供定义。

### 数据表访问

```rust
// TableReader<T> 按 key 查询
fn read_table(reader: TableReader<TbItem>) {
    let item: Arc<Item> = reader.get_row("item_id")?;
}
```

加载流程: `DataTablePlugin` → `TableLoadingState::Wait` → `Loading` → `Loaded` → `TablesLoadedEvent` (Message)。
`AllAssetBarrier` 管理命名 barrier，`TablesBarrierStatus` 跟踪表加载状态。

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

### atom_ability (EffectGraph 系统)

```rust
// 子模块: graph/, ability/, buff/, attribute/, stateset/
// AbilitySubsystemPlugin 聚合所有子插件

app.add_plugins(AbilitySubsystemPlugin);
// 内部注册: AbilityPlugin, BuffPlugin, EffectGraphPlugin,
//           EffectNodePlugin, EffectNodeAbilityEntryPlugin

// 核心概念
// EffectGraph: 节点图执行引擎 (blackboard, context, events, executor)
// StateLayerTagRegistry: LayerTag 状态注册表 (由 TbLayerTag 配置表初始化)
// StateSet: 运行时状态集合

// 典型用法
AbilityBundle::new(ability_row, &state_registry);  // 从配置行构建
BuffBundle::new(buff_row, &state_registry);
```

### 异步模式

- 自定义 `Future` 实现 (`AssetBarrier`): `Arc<AtomicUsize>` 引用计数 + `Mutex<Option<Waker>>` 唤醒
- 双检查模式防丢失唤醒
- 无 `tokio`/`async-std` — 全部基于 `std`，兼容 Bevy 异步执行器

### 惰性初始化

- `once_cell::sync::OnceCell` — 静态路径 (`ProjectPaths`)
- `std::sync::OnceLock` — 日志文件名
- 首次访问初始化，后续零开销

## 测试

- 使用 Rust 内置 `#[test]`，内联于 `#[cfg(test)] mod tests`
- 运行: `cargo test --workspace`
- 暂无 CI，手动运行 clippy + test
