# Bevy 0.19 迁移速查

## Render / Compute

| 0.18 | 0.19 | 备注 |
|------|------|------|
| `render_graph::Node` (update + run) | `RenderContext` system param | 直接在 system 中拿 encoder |
| `RenderGraphApp::add_render_graph_node()` | 删除 | 改用 system + `Render` schedule |
| `ViewNodeRunner<T>` | 删除 | 改用 system |
| `RenderLabel` | 删除 | 改用 system ordering |
| `device.inner()` | `device.wgpu_device()` | 返回 `&wgpu::Device` |

## Bind Group / Pipeline

| 0.18 | 0.19 | 备注 |
|------|------|------|
| `BindGroupLayoutEntry { ... }` 手写 | `BindGroupLayoutEntries::sequential(stage, (...))` | builder pattern |
| 手动 `uniform_buffer::<T>(false)` | `binding_types::uniform_buffer::<T>(false)` | 在 `render_resource::binding_types` |
| `storage_buffer::<Vec<T>>(false)` | `binding_types::storage_buffer::<Vec<T>>(false)` | 同上 |
| `min_binding_size: Some(size)` | `min_binding_size: NonZero::new(size)` | `ShaderSize` → `NonZeroU64` |
| `PipelineLayoutDescriptor` | `BindGroupLayoutDescriptor` | 直接用在 `ComputePipelineDescriptor.layout` |
| `entry_point: "main".into()` | `entry_point: Some("main".into())` | `Option<Cow<'static, str>>` |
| `queue_compute_pipeline(...)` | 不变 | 参数从 `PipelineLayoutDescriptor` 改为 `BindGroupLayoutDescriptor` |

## ECS / Resources

| 0.18 | 0.19 | 备注 |
|------|------|------|
| `Commands::despawn_recursive()` | `Commands::despawn()` | despawn 自动递归 |
| `MessageWriter::send()` | `MessageWriter::write()` | |
| `RenderApp` | 不变 | `app.sub_app_mut(RenderApp)` 仍可用 |
| `ExtractResource` | 不变 | 但 `Resource` 现在是 `Component` 的 subtrait |
| `RenderStartup` | 新 | render world 一次性初始化 |
| `Render` | 新 | 替代旧 render graph node，render world systems |

## Asset / 数据加载（bevy_common_assets）

| 0.18 / 旧 | 0.19 | 备注 |
|------|------|------|
| — | `bevy_common_assets 0.17.0` | 原生兼容 Bevy 0.19（依赖 bevy_app/bevy_asset/bevy_reflect ^0.19）；9 格式 feature 全开（json/ron/toml/yaml/csv/msgpack/cbor/xml/postcard），0 默认 feature |
| — | `XxxAssetPlugin::<A>::new(&["json"])` | 泛型插件：`A: for<'de> Deserialize<'de> + Asset`；同类型可注册多格式插件（扩展名路由）；plugin build 内部自带 `init_asset::<A>()` |
| — | `AssetLoader` trait 含 `TypePath` supertrait | 0.18 新增；loader 类型需实现 TypePath |
| `LoadContext::load_direct` / `NestedLoader` | `LoadContext::load_builder()` | 子资产加载 API 重构（0.18） |
| `AssetEvent::Loaded` 即资产可用 | `AssetEvent::LoadedWithDependencies { id }` | 本体 + 全部递归依赖就绪信号；`event.is_loaded_with_dependencies(id)` 便捷判断；注册了 loader 的 asset 类型该事件由 `init_asset` 自动注册（`AssetEventSystems`） |
| — | `AssetServer::get_id_handle::<A>(id)` | `AssetId<A>` → `Handle<A>`（事件回调中取句柄用）；`Assets::get(id)` 直接接受 `impl Into<AssetId<A>>` |

## Mesh / Asset

| 0.18 | 0.19 | 备注 |
|------|------|------|
| `PrimitiveTopology` from `bevy::render::mesh` | `bevy::mesh::PrimitiveTopology` | 公开但路径变 |
| `Indices::U32(...)` | `bevy::mesh::Indices::U32(...)` | 同上 |
| `RenderAssetUsages::default()` | `bevy::asset::RenderAssetUsages::default()` | 从 bevy_render 移到 bevy_asset |

## Camera / Input

| 0.18 | 0.19 | 备注 |
|------|------|------|
| 第三方 flycam | `FreeCamera` + `FreeCameraPlugin` | feature `bevy_camera_controller` + `free_camera`; 右键旋转，WASD/QE 移动 |
| `Input<T>` | `ButtonInput<T>` | |
| `MouseMotion` event (EventReader) | `AccumulatedMouseMotion` resource | 每帧累积 delta → `acc_motion.delta` |
| `MouseWheel` event (EventReader) | `AccumulatedMouseScroll` resource | 同上，`acc_scroll.delta` |

## Material

| 0.18 | 0.19 | 备注 |
|------|------|------|
| — | `StandardMaterial::cull_mode: None` | 禁用背面剔除 |
| — | `StandardMaterial::double_sided: true` | 仅影响光照(双面法线)，**不关剔除**，要配合 `cull_mode: None` |

## wgpu

| 0.18 | 0.19 | 备注 |
|------|------|------|
| `wgpu::Maintain::Wait` | `PollType::Wait { submission_index: None, timeout: None }` | 来自 `wgpu_types`，通过 `bevy::render::render_resource::PollType` |
| `wgpu::MapMode::Read` | `MapMode::Read` (from bevy re-export) | `bevy::render::render_resource::MapMode` |
| `wgpu 27.0` | `wgpu 29.0` | |
