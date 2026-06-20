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

## Mesh / Asset

| 0.18 | 0.19 | 备注 |
|------|------|------|
| `PrimitiveTopology` from `bevy::render::mesh` | `bevy::mesh::PrimitiveTopology` | 公开但路径变 |
| `Indices::U32(...)` | `bevy::mesh::Indices::U32(...)` | 同上 |
| `RenderAssetUsages::default()` | `bevy::asset::RenderAssetUsages::default()` | 从 bevy_render 移到 bevy_asset |

## wgpu

| 0.18 | 0.19 | 备注 |
|------|------|------|
| `wgpu::Maintain::Wait` | `PollType::Wait { submission_index: None, timeout: None }` | 来自 `wgpu_types`，通过 `bevy::render::render_resource::PollType` |
| `wgpu::MapMode::Read` | `MapMode::Read` (from bevy re-export) | `bevy::render::render_resource::MapMode` |
| `wgpu 27.0` | `wgpu 29.0` | |
