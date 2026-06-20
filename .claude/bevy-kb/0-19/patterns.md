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

## Render World 资源管理 (0.19 已验证)

`ExtractResourcePlugin` 在 0.19 pipelined rendering 中不可用。render world 资源直接 insert：

```rust
// ❌ 不工作: ExtractResourcePlugin::<T>::default()
// ✅ 直接插入到 render world
let render_app = app.sub_app_mut(RenderApp);
render_app.insert_resource(TerrainSetting::default());
render_app.init_resource::<TerrainChunkMeshBuffers>();
```

已在 compute pipeline 中验证。

## WGSL Storage Buffer 访问修饰符 (0.19 已验证)

`var<storage>` 无访问修饰符时**默认为 `read`**，不是 `read_write`：

```wgsl
// ❌ 只读: var<storage> data: array<u32>;
// ✅ 读写: var<storage, read_write> data: array<u32>;
```

## WGSL Uniform Struct 对齐规则 (0.19 已验证)

WGSL `uniform` 地址空间的 struct 与 Rust `#[repr(C)]` 布局不同：

```
vec3<f32> 对齐: 16 字节 (不是 12)
struct 总大小: 16 的倍数 (不是按字段紧凑排列)
```

| 字段 | Rust offset | WGSL uniform offset |
|------|------------|-------------------|
| `chunk_min: vec3<f32>` | 0 | 0 (12B + 4B pad) |
| `voxel_size: f32` | 12 | 16 |
| total | 36 | 48 |

必须在 Rust struct 中添加显式 padding 字段以匹配 WGSL 布局。

## Buffer Usage 约束 (wgpu 29, 0.19 已验证)

- `MAP_READ` 只能与 `COPY_DST` 组合（不能与 `COPY_SRC` 或 `STORAGE`）
- CPU 读回 GBU buffer 需要 staging buffer: compute writes → `STORAGE|COPY_SRC`, then copy to `COPY_DST|MAP_READ`
- `queue.write_buffer()` 需要 `COPY_DST` flag
