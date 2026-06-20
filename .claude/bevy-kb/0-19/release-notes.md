# Bevy 0.19 — 与 atom_terrain 相关的变更

## Render Graph → Systems (关键变更)

0.19 将 render graph 从独立 Node trait 改成了普通 ECS systems。
旧模式（`render_graph::Node::update()` + `run()`）已删除。

**影响**: atom_terrain 的 compute dispatch 从 `TerrainMeshComputeNode` 改为 `terrain_compute_system` system。

**新 API**:
```rust
// system 中直接拿 command encoder
fn my_system(mut render_context: RenderContext) {
    let mut encoder = render_context.command_encoder();
    let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
    cpass.set_pipeline(pipeline);
    cpass.dispatch_workgroups(x, y, z);
}
app.add_systems(Render, my_system);
```

**Schedule 标签**: `RenderStartup`（一次性初始化）、`Render`（每帧）。

## Bind Group API 重构

`BindGroupLayoutEntry` 手写改为 builder pattern:

```rust
let entries = BindGroupLayoutEntries::sequential(
    ShaderStages::COMPUTE,
    (
        uniform_buffer::<TerrainChunkInfo>(false),
        storage_buffer::<Vec<f32>>(false),
        storage_buffer::<Vec<TerrainChunkVertex>>(false),
    ),
);
let desc = BindGroupLayoutDescriptor::new("label", &entries);
```

注意:
- `uniform_buffer` / `storage_buffer` 在 `bevy::render::render_resource::binding_types`
- `min_binding_size` 类型从 `u64` 改为 `Option<NonZeroU64>`
- Compute pipeline 的 `layout` 字段接受 `Vec<BindGroupLayoutDescriptor>`（不再需要单独的 PipelineLayout）

## Resources as Components

`Resource` 现在是 `Component` 的 subtrait。`#[derive(Resource)]` 同时实现 `Component`。
- `Res<T>` / `ResMut<T>` 不变
- `ExtractResourcePlugin` 不变
- 类型不能再同时是 Resource 和 Component（会冲突）

## wgpu 27 → 29

- `Maintain::Wait` → `PollType::Wait { submission_index: None, timeout: None }`
- `device.inner()` / `device.wgpu()` → `device.wgpu_device()`
- `MapMode::Read` 路径: `bevy::render::render_resource::MapMode`

## Mesh API

```rust
// 0.18
Mesh::new(PrimitiveTopology::TriangleList)

// 0.19
Mesh::new(
    bevy::mesh::PrimitiveTopology::TriangleList,
    bevy::asset::RenderAssetUsages::default(),
)
```

`PrimitiveTopology` 和 `Indices` 从 `bevy::render::mesh` 移到 `bevy::mesh`。

## ECS 微小变更

- `Commands::despawn_recursive()` → `Commands::despawn()`（自动递归）
- `MessageWriter::send()` → `MessageWriter::write()`
- `#[derive(Resource)]` 类型不能再 derive `Component`
