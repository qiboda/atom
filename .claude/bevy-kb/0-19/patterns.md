# Bevy 0.19 — atom_terrain 已验证的 API 模式

## Compute Pipeline 完整模式

```rust
use bevy::render::{
    render_resource::{binding_types::*, *},
    renderer::{RenderContext, RenderDevice, RenderQueue},
    Render, RenderApp, RenderStartup,
};

// 1. 一次性 pipeline 初始化 (RenderStartup)
fn init_compute_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
) {
    let entries = BindGroupLayoutEntries::sequential(
        ShaderStages::COMPUTE,
        (
            uniform_buffer::<ChunkInfo>(false),     // binding 0
            storage_buffer::<Vec<f32>>(false),       // binding 1
            storage_buffer::<Vec<u32>>(false),       // binding 2
        ),
    );
    let desc = BindGroupLayoutDescriptor::new("label", &entries);
    let bgl = render_device.create_bind_group_layout("label", &entries);

    let pipeline_id = pipeline_cache.queue_compute_pipeline(
        ComputePipelineDescriptor {
            label: Some("my_pass".into()),
            layout: vec![desc.clone()],
            shader: asset_server.load("shaders/xxx.wgsl"),
            ..default()
        }
    );

    commands.insert_resource(MyPipeline { bind_group_layout: bgl, pipeline: pipeline_id });
}

// 2. 注册 plugin
app.sub_app_mut(RenderApp)
    .add_systems(RenderStartup, init_compute_pipeline)
    .add_systems(Render, my_compute_system);
```

## 每帧 dispatch 模式

```rust
fn my_compute_system(
    mut render_context: RenderContext,
    pipeline_cache: Res<PipelineCache>,
    pipeline: Res<MyPipeline>,
    // ... your resources
) {
    let mut encoder = render_context.command_encoder();
    let Some(cp) = pipeline_cache.get_compute_pipeline(pipeline.pipeline) else {
        return;
    };

    let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
    cpass.set_pipeline(cp);
    cpass.set_bind_group(0, &bind_group, &[]);
    cpass.dispatch_workgroups(x, y, z);
    // cpass is dropped here implicitly
}
```

注意: `encoder.begin_compute_pass` 返回的 `ComputePass` 在 drop 时自动结束。不需要显式 `mem::drop`，scope 离开即可。

## GPU Buffer 读回模式

```rust
fn poll_and_read(device: &RenderDevice, buffer: &Buffer, size: u64) -> Option<Box<[u8]>> {
    let slice = buffer.slice(..size);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(MapMode::Read, move |_| { tx.send(()).ok(); });
    device.wgpu_device().poll(PollType::Wait { submission_index: None, timeout: None });
    if rx.recv().is_ok() {
        let view = slice.get_mapped_range();
        let data = view.to_vec().into_boxed_slice();
        drop(view);
        buffer.unmap();
        Some(data)
    } else {
        None
    }
}
```

注意:
- `PollType` 从 `bevy::render::render_resource` re-export
- `MapMode` 同上
- `device.wgpu_device()` 返回 `&wgpu::Device`（不是 `inner()`）

## Mesh 创建模式

```rust
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;

let mut mesh = Mesh::new(
    PrimitiveTopology::TriangleList,
    RenderAssetUsages::default(),
);
mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
mesh.insert_indices(Indices::U32(indices));
```

## ExtractResource 同步模式

```rust
// main world 和 render world 之间同步
#[derive(Resource, Clone, ExtractResource)]
pub struct MySyncData { ... }

// main world
app.insert_resource(MySyncData { ... });

// render world 自动获得 clone 的副本
fn render_system(data: Res<MySyncData>) { ... }
```

注意: 0.19 中 `Resource` 需要 `Mutability = Mutable` bound 才能使用 `ResMut`:

```rust
// 如果泛型需要 ResMut
fn generic_system<R: Resource<Mutability = Mutable>>(mut res: ResMut<R>) { ... }
```
