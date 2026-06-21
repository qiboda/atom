# 0.18 → 0.19 迁移要点（atom_terrain 相关）

## 关键 Breaking Changes

1. **Render Graph 被 System 替代** — 最大的架构变化
2. **Bind Group API 改为 builder pattern** — `BindGroupLayoutEntries::sequential()`
3. **Resources 成为 Components** — `#[derive(Resource)]` 同时 derive Component
4. **wgpu 27→29** — PollType, MapMode 路径变化
5. **Mesh API 签名变化** — 需要 `RenderAssetUsages` 参数

## 迁移步骤（已验证）

### 1. 删除 render graph node

删除所有 `impl render_graph::Node for ...` 代码，改为 system:

```rust
// 旧: node.rs 中的 update() + run()
impl render_graph::Node for TerrainMeshComputeNode {
    fn update(&mut self, world: &mut World) { ... }
    fn run(&self, graph: &mut RenderGraphContext, ...) { ... }
}

// 新: gpu.rs 中的 system
fn terrain_compute_system(mut render_context: RenderContext, ...) {
    let mut encoder = render_context.command_encoder();
    // dispatch ...
}
```

### 2. 重写 bind group 创建

```rust
// 旧
let entries = vec![BindGroupLayoutEntry { binding: 0, ... }, ...];
let bgl = render_device.create_bind_group_layout("label", &entries);

// 新
let entries = BindGroupLayoutEntries::sequential(
    ShaderStages::COMPUTE,
    (uniform_buffer::<T>(false), storage_buffer::<Vec<U>>(false), ...),
);
```

### 3. 更新 pipeline 创建

```rust
// 旧: ComputePipelineDescriptor { layout: vec![pipeline_layout], ... }
// 新: ComputePipelineDescriptor { layout: vec![bind_group_descriptor], entry_point: Some("main".into()), ... }
```

### 4. 适配 wgpu API

```rust
// 旧
device.inner().poll(wgpu::Maintain::Wait);
buffer.slice(..).map_async(wgpu::MapMode::Read, callback);

// 新
device.wgpu_device().poll(PollType::Wait { submission_index: None, timeout: None });
buffer.slice(..).map_async(MapMode::Read, callback);
```

### 5. Mesh 创建

```rust
// 旧
Mesh::new(PrimitiveTopology::TriangleList)

// 新
use bevy::mesh::PrimitiveTopology;
use bevy::asset::RenderAssetUsages;
Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
```
