# BSN Patterns — 实际项目映射

本文记录 BSN 与 `atom_terrain` 项目的映射关系，作为格式设计的验证样本。

## 模式 1: Observer-Driven Compute Pipeline

**场景**: 摄像机移动 → Observer 更新 → Render World GPU compute → mesh 回传 main world。

**Rust 端** (`global_compute.rs`):
```rust
app.add_systems(Update, (
    update_observer_from_camera,
    handle_global_mesh_data,
    debug_keyboard_toggle,
    apply_debug_wireframe,
).chain());

render_app.add_systems(Render, global_compute_system);
```

**BSN 映射**:
```
// global-terrain.bsn
plugin GlobalTerrainPlugin {
    // ── Main World ──
    systems: [
        system update_observer_from_camera {
            schedule: Update
            params: [Query<&Transform, With<Camera3d>>, ResMut<TerrainObserver>]
        },
        system handle_global_mesh_data {
            schedule: Update
            params: [Commands, Res<TerrainChunkMeshReceiver>, Res<TerrainDebugConfig>,
                     ResMut<Assets<Mesh>>, ResMut<Assets<StandardMaterial>>,
                     Local<Option<Entity>>, ResMut<GlobalTerrainMaterial>]
            after: [update_observer_from_camera]
        },
        system apply_debug_wireframe {
            schedule: Update
            params: [Res<TerrainDebugConfig>, Query<Entity, With<GlobalTerrainMesh>>, Commands]
            after: [handle_global_mesh_data]
        },
    ]

    // ── Render World (crossbeam) ──
    // 注: render app 系统用 Render schedule，通过 TerrainChunkMeshSender 通信
    systems: [
        system global_compute_system {
            schedule: Render
            params: [Res<TerrainObserver>, ResMut<GlobalComputeState>,
                     Res<RenderDevice>, Res<RenderQueue>]
        },
    ]

    resources: [TerrainSetting, TerrainObserver, TerrainDebugConfig,
                TerrainChunkMeshReceiver, GlobalTerrainMaterial, GlobalComputeState]
}
```

## 模式 2: State-Driven System Chaining

**场景**: 多阶段 compute pipeline 用 `TerrainState` 驱动 system set 链式执行。

**Rust 端** (旧 per-chunk 系统):
```rust
app.configure_sets(Update, (
    TerrainSystems::ChunkLoader,
    TerrainSystems::ApplyCSG,
    TerrainSystems::GenerateChunk,
).chain().run_if(in_state(TerrainState::GenerateTerrainMesh)));
```

**BSN 映射**:
```
state TerrainState {
    variants: [None, LoadAssets, GenerateTerrainRegion, GenerateTerrainMesh]
}

system_set TerrainSystems {
    chain: [chunk_loader, apply_csg, generate_chunk]
    run_if: in_state(TerrainState::GenerateTerrainMesh)
}
```

## 模式 3: Crossbeam Channel (Main↔Render)

**场景**: Render world 生成 mesh，通过 crossbeam channel 发送到 main world 渲染。

**Rust 端**:
```rust
let (mesh_tx, mesh_rx) = crossbeam::channel::unbounded();
app.insert_resource(TerrainChunkMeshReceiver(mesh_rx));
render_app.insert_resource(TerrainChunkMeshSender(mesh_tx));
```

**BSN 映射**:
```
// 跨世界通信用 channel 标记
channel mesh_channel {
    ty: TerrainChunkMeshData
    sender: Render   // render world → main world
    receiver: Main
}
```

## 模式 4: GPU Compute Pipeline State Machine

**场景**: GPU compute 4-frame 延迟管线用 `pass` 字段手动推进。

**Rust 端** (`GlobalComputeState`):
```rust
pub struct GlobalComputeState {
    pub pass: u32,          // 0=idle, 1=staging_copy, 2=readback
    pub last_grid_min: Vec3,
    pub grid_min: Vec3,
    pub rebuild_triggered: bool,
}
```

**BSN 映射**:
```
resource GlobalComputeState {
    pass: 0,                          // 0=idle, 1=staging_copy, 2=readback
    last_grid_min: Vec3(0, 0, 0),
    grid_min: Vec3(0, 0, 0),
    rebuild_triggered: false,
    last_force_rebuild: 0,
}
```

## 模式 5: GPU Buffer Pool (多 pass 共享)

**场景**: 6 个 compute passes 共享 buffer pool — vertices, indices, cross, counters, voxel_alloc。

**Rust 端** (`GlobalMeshPool`):
```rust
pub struct GlobalMeshPool {
    pub vertices: Buffer,      // pass 3 write, readback
    pub indices: Buffer,       // pass 4 write, readback
    pub cross: Buffer,         // pass 1 write, pass 2/3/4 read
    pub counters: Buffer,      // pass 2/4 atomic write
    pub voxel_alloc: Buffer,   // pass 2 write, pass 3/4 read
    pub grid_size: u32,
}
```

**BSN 映射**:
```
// GPU buffer pool — 不是 ECS 资源，用 buffer_pool 标记
buffer_pool GlobalMeshPool {
    buffers: [
        buffer vertices    { usage: [Storage, CopySrc], pass_read: [3, readback], pass_write: [3] },
        buffer indices     { usage: [Storage, CopySrc], pass_read: [4, readback], pass_write: [4] },
        buffer cross       { usage: [Storage],          pass_read: [1,2,3,4],  pass_write: [1] },
        buffer counters    { usage: [Storage],          pass_read: [2,4,readback], pass_write: [2,4] },
        buffer voxel_alloc { usage: [Storage],          pass_read: [3,4], pass_write: [2] },
    ]
    grid_size: 50
}
```

## BSN 的边界

以下内容**不在** BSN 描述范围内，属于实现细节：

- 函数体逻辑（shader 代码、CPU 算法）
- GPU pipeline layout / bind group layout
- 异步回调（`map_async` closure）
- 瞬时中间变量（`encoder.clear_buffer` 等）
- Benchmark / profiling 代码
