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

## GPU Buffer 读回模式（非阻塞，Render 系统安全）

⚠️ **不要用 `PollType::Wait`** — 会阻塞渲染线程导致 swap chain timeout。

```rust
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

// 1. Staging copy（Render 系统中，拿到 encoder 后）
let staging = device.create_buffer(&BufferDescriptor {
    label: Some("staging"),
    size,
    usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
    mapped_at_creation: false,
});
encoder.copy_buffer_to_buffer(&src_gpu_buffer, 0, &staging, 0, size);

// 2. 下一个 frame（copy 已被 GPU 执行），发起 async map
let mapped = Arc::new(AtomicBool::new(false));
let flag = mapped.clone();
staging.slice(..).map_async(MapMode::Read, move |result| {
    if result.is_ok() { flag.store(true, Ordering::Release); }
});
let _ = device.wgpu_device().poll(PollType::Poll); // 非阻塞触发回调

// 3. 再下一个 frame（或同 frame 如果 poll 触发成功），检查并读取
if mapped.load(Ordering::Acquire) {
    let view = staging.slice(..).get_mapped_range();
    let data: &[MyType] = bytemuck::cast_slice(&view);
    // ... 使用 data ...
    drop(view);
    staging.unmap();
}
```

时序: `dispatch → (1帧) → copy → (1帧) → map → read`。
同一帧内 dispatch+copy 读到全零（GPU 还没执行 dispatch）。
不要在同一帧内 copy+map+read（GPU 还没执行 copy）。

**关键 API 路径**:
- `MapMode` → `bevy::render::render_resource::MapMode`
- `PollType` → `bevy::render::render_resource::PollType`
- `device.wgpu_device()` → `&wgpu::Device`（不是 `inner()`）

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

## Game Framework: Mesh + Material Components (0.19)

在 Bevy 0.19 中，`Mesh3d` 和 `MeshMaterial3d` 替换了旧的 `Material3d`：

```rust
// ❌ 旧 API (0.18-): Material3d::from(Color::srgb(...))
// ❌ 旧 API: Mesh3d::from(mesh)

// ✅ 0.19 正确用法：
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.5).mesh().ico(3).unwrap())),
        MeshMaterial3d(materials.add(Color::srgb(0.2, 0.6, 1.0))),
        Transform::from_xyz(0.0, -20.0, 0.0),
    ));
}
```

重点:
- `Mesh3d(pub Handle<Mesh>)` — 不是 `From<Mesh>`
- `MeshMaterial3d<M: Material>(pub Handle<M>)` — 不是 `Material3d::from(Color)`
- `Sphere::new(0.5).mesh().ico(3).unwrap()` 创建 ICO 球体 mesh

## Game Framework: Orthographic Camera Setup (0.19)

```rust
// ❌ OrthographicProjection 不实现 Default trait（只有 FromWorld）

// ✅ 使用 default_3d() 构造器：
commands.spawn((
    Camera3d::default(),
    Projection::from(OrthographicProjection {
        scaling_mode: ScalingMode::WindowSize,
        ..OrthographicProjection::default_3d()
    }),
    Transform::from_xyz(0.0, -10.0, 0.0).looking_at(Vec3::new(0.0, -20.0, 0.0), Vec3::Z),
));
```

类型路径:
- `ScalingMode` → `bevy::camera::ScalingMode`（不在 prelude 中）
- `OrthographicProjection::default_3d()` → 标准 3D 正交投影构造器
- `OrthographicProjection::default_2d()` → 2D 版本

## #![deny(missing_docs)] 与模块声明

当 crate 设置了 `#![deny(missing_docs)]` 时，`pub mod` 声明也需要文档注释：

```rust
// ❌ error: missing documentation for a module
pub mod camera;

// ✅ 必须加文档注释
/// 俯视角摄像机系统。
pub mod camera;
```
