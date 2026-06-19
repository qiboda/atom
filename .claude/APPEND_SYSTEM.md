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

### ECS 依赖注入

```rust
// Resource 注入
app.insert_resource(MyResource::default());

// 自定义 SystemParam
fn my_system(reader: TableReader<TbItem>, barrier: AllAssetBarrier) { .. }

// ExtractResource — 渲染世界同步
#[derive(Resource, Clone, ExtractResource)]
struct TerrainSetting { .. }
```

### GPU Buffer 生命周期

```rust
// atom_render 显式管理
let mut buf: SharedStorageBuffer<T> = ..;
buf.reserve(count);   // 分配
buf.write(&data);     // 写入
buf.push();           // 提交
buf.clear();          // 重置
```

### Shader 资源管理

通过 `atom_shader_lib` 宏自动生成 Shader 加载 Plugin:

```rust
atom_shaders_plugin!(
    MyShadersPlugin,
    "noise/core/fbm.wgsl" -> fbm,
    "noise/core/open_simplex.wgsl" -> open_simplex,
);
// 生成 MyShadersPlugin: Plugin (自动 load_asset + insert_resource)
// 生成 MyShaders Resource: { fbm: Handle<Shader>, open_simplex: Handle<Shader> }
```

Shader 路径相对于 `assets/shaders/`。Shader 修改后 `file_watcher` 自动热重载，不需要重启。

### LayerTag 分层标签

```rust
let tag = LayerTag::new("ability.stun.fire");

tag.exact_match(&other);      // 完全相等
tag.partial_match(&other);    // 前缀匹配 (a.b 匹配 a.b.c)
tag.same_prefix(&other);      // 同前缀

// 两种容器 (均为 Bevy Component)
SingleLayerTagContainer  // HashSet<LayerTag> — 去重语义
CountLayerTagContainer   // Vec<CountLayerTag> — 引用计数语义
```

LayerTag 需先通过 `LayerTagRegistry` 注册才能使用。配置表 `TbLayerTag` 提供定义。

### 数据表访问

```rust
fn read_table(reader: TableReader<TbItem>) {
    let item: Arc<Item> = reader.get_row("item_id")?; // MapTable 按键查询
}
```

数据表加载流程: `DataTablePlugin` → `Wait` → `Loading` → `Loaded` → `TablesLoadedEvent`。

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
