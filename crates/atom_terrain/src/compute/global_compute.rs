//! 全局 Edge Graph DC 管线 — 观察者中心，单次 QEF 求解，无 chunk 边界。
//!
//! ## 管线
//!
//! ```text
//! Frame N:   clear cross/counters → dispatch pass0→1→2→3→4 (同一 encoder)
//! Frame N+1: staging copy → 等 GPU 执行
//! Frame N+2: readback counters → vertices → indices → build Mesh → send to main
//! ```
//!
//! ## 与 per-chunk 管线的区别
//!
//! - 无 chunk 边界 → 无 shell，无 seam
//! - 全局 edge_id → 每条 edge 只求解一次
//! - GPU atomic counters → 精确分配 vertex/index slot
//! - 观察者驱动 → 只在观察者移动时触发重建

use std::sync::{Arc, atomic::AtomicBool};

use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_resource::{
            BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor,
            BindGroupLayoutEntries, Buffer, BufferDescriptor, BufferUsages,
            CachedComputePipelineId, ComputePassDescriptor, ComputePipelineDescriptor, MapMode,
            PipelineCache, ShaderStages, ShaderType, binding_types::*,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
    },
};

use super::{global_pool::GlobalMeshPool, types::TerrainChunkVertex};
use crate::{mesh::TerrainChunkMeshData, mesh::TerrainChunkMeshSender};

/// 观察者位置，主世界每帧更新，渲染世界自动提取。
#[derive(Resource, Clone, Debug, Default, ExtractResource)]
pub struct TerrainObserver {
    /// 观察者世界坐标（通常为相机位置）
    pub position: Vec3,
    /// 强制重建计数器：主世界递增 → 渲染世界检测变化 → 触发重建
    pub force_rebuild: u32,
}

/// 全局 compute 管线资源
#[derive(Resource)]
pub struct GlobalComputePipeline {
    /// uniform buffer（每轮重建时更新 grid_min）
    pub uniform_buffer: Buffer,
    /// 全局 bind group（8 个 binding: 0..=7）
    pub bind_group: BindGroup,
    /// bind group layout
    pub bind_group_layout: BindGroupLayout,
    /// Pass 0: SDF 填充 (sdf_fill.wgsl)
    pub pass0: CachedComputePipelineId,
    /// Pass 1: 全局 edge 检测 (edge_detect.wgsl)
    pub pass1: CachedComputePipelineId,
    /// Pass 2: atomic vertex 分配 (vertex_alloc.wgsl)
    pub pass2: CachedComputePipelineId,
    /// Pass 3: 全局 QEF 求解 (qef_solve.wgsl)
    pub pass3: CachedComputePipelineId,
    /// Pass 4: quad 索引生成 (index_build.wgsl)
    pub pass4: CachedComputePipelineId,
    /// Pass 5: 填充 indirect draw command (fill_indirect.wgsl)
    pub pass5: CachedComputePipelineId,
}

/// 全局管线的当前阶段
///
/// 状态机:
///   0 (idle) → 触发重建 → 清 buffer → dispatch pass0-4 → 1
///   1 (dispatched, 等 GPU) → staging copy → 2
///   2 (staging, 等 GPU copy) → (等一帧) → map → readback → 3
///   3 (readback done) → build mesh → send → 0 (回到 idle)
/// 全局管线的当前阶段
///
/// 状态机:
///   0 (idle) → 触发重建 → 清 buffer → dispatch pass0-5 → 1
///   1 (dispatched, 等 GPU) → (readback_enabled ? staging → 2 : 0)
///   2 (staging, 等 GPU copy) → (等一帧) → map → readback → 0
#[derive(Resource, Default)]
pub struct GlobalComputeState {
    /// 当前 pass: 0=idle, 1=dispatched, 2=staging
    pub pass: u32,
    /// 上次重建时的 world-aligned grid_min
    pub last_grid_min: Vec3,
    /// 当前 grid_min（用于 mesh world_offset）
    pub grid_min: Vec3,
    /// 上次是否已触发重建（避免 idle 帧重复触发）
    pub rebuild_triggered: bool,
    /// 上次 force_rebuild 计数器值
    pub last_force_rebuild: u32,
    /// 自上次重建以来有有效的 GPU 地形数据（供 indirect draw 使用）
    pub has_valid_data: bool,
    /// 是否启用 CPU readback（用于碰撞/导航数据）；关闭时跳过 staging & readback
    pub readback_enabled: bool,
}

/// GPU→CPU readback staging
struct GlobalStaging {
    vertices: Buffer,
    indices: Buffer,
    counters: Buffer,
    voxel_alloc: Buffer,   // fixed slot → compact index mapping
    vertex_cap: u64,       // vertex buffer 总字节数
    index_cap: u64,        // index buffer 总字节数
    voxel_alloc_size: u64, // voxel_alloc buffer 总字节数
    mapped: Arc<AtomicBool>,
    map_started: bool,
    grid_min: Vec3,
}

/// 全局 staging readback（单例，不是 HashMap）
#[derive(Resource, Default)]
pub struct GlobalStagingState {
    staging: Option<GlobalStaging>,
}

// ── WGSL GlobalUniforms 对应 ──

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable, ShaderType)]
struct GlobalUniformsGpu {
    grid_min: [f32; 3],
    pad0: u32, // → 16
    voxel_size: f32,
    grid_size: u32,
    pad1: [u32; 2],     // → 32
    neighbor_mask: u32, // global管线不使用，占位
    pad_neighbor: u32,  // → 40
    seed: u32,          // noise seed (offset 40)
    pad2: u32,          // → 48
}

impl GlobalUniformsGpu {
    fn new(grid_min: Vec3, voxel_size: f32, grid_size: u32) -> Self {
        Self {
            grid_min: grid_min.to_array(),
            pad0: 0,
            voxel_size,
            grid_size,
            pad1: [0; 2],
            neighbor_mask: 0,
            pad_neighbor: 0,
            seed: 42,
            pad2: 0,
        }
    }
}

/// 初始化全局 compute pipeline（RenderStartup）。
pub fn init_global_compute_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    pool: Res<GlobalMeshPool>,
    queue: Res<RenderQueue>,
) {
    let entries = BindGroupLayoutEntries::sequential(
        ShaderStages::COMPUTE,
        (
            uniform_buffer::<GlobalUniformsGpu>(false),
            storage_buffer::<Vec<f32>>(false), // 1: density
            storage_buffer::<Vec<u32>>(false), // 2: cross
            storage_buffer::<Vec<u32>>(false), // 3: voxel_alloc
            storage_buffer::<Vec<TerrainChunkVertex>>(false), // 4: vertices
            storage_buffer::<Vec<u32>>(false), // 5: counters (atomic)
            storage_buffer::<Vec<u32>>(false), // 6: indices
            storage_buffer::<Vec<u32>>(false), // 7: indirect draw command
        ),
    );
    let desc = BindGroupLayoutDescriptor::new("global_bgl", &entries);
    let bgl = render_device.create_bind_group_layout("global_bgl", &entries);

    let uniform = GlobalUniformsGpu::new(Vec3::ZERO, pool.voxel_size(), pool.grid_size);
    let ub = render_device.create_buffer(&BufferDescriptor {
        label: Some("global_uniform"),
        size: size_of::<GlobalUniformsGpu>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&ub, 0, bytemuck::bytes_of(&uniform));

    let zero: [u32; 4] = [0; 4];
    queue.write_buffer(&pool.counters, 0, bytemuck::bytes_of(&zero));

    let bg = render_device.create_bind_group(
        "global_bg",
        &bgl,
        &[
            BindGroupEntry {
                binding: 0,
                resource: ub.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: pool.sdf.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: pool.cross.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 3,
                resource: pool.voxel_alloc.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 4,
                resource: pool.vertices.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 5,
                resource: pool.counters.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 6,
                resource: pool.indices.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 7,
                resource: pool.indirect.as_entire_binding(),
            },
        ],
    );

    let mk = |label: &'static str, path: &'static str| {
        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(label.into()),
            layout: vec![desc.clone()],
            shader: asset_server.load(path),
            ..default()
        })
    };

    commands.insert_resource(GlobalComputePipeline {
        uniform_buffer: ub,
        bind_group: bg,
        bind_group_layout: bgl,
        pass0: mk("global_sdf", "shaders/terrain/compute/sdf_fill.wgsl"),
        pass1: mk("global_edge", "shaders/terrain/compute/edge_detect.wgsl"),
        pass2: mk("global_alloc", "shaders/terrain/compute/vertex_alloc.wgsl"),
        pass3: mk("global_qef", "shaders/terrain/compute/qef_solve.wgsl"),
        pass4: mk("global_index", "shaders/terrain/compute/index_build.wgsl"),
        pass5: mk(
            "global_indirect",
            "shaders/terrain/compute/fill_indirect.wgsl",
        ),
    });
}

/// 每帧执行的全局 compute 调度系统。
#[allow(clippy::too_many_arguments)]
pub fn global_compute_system(
    mut render_context: RenderContext,
    pipeline_cache: Res<PipelineCache>,
    pipeline: Res<GlobalComputePipeline>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    pool: Res<GlobalMeshPool>,
    observer: Option<Res<TerrainObserver>>,
    mut state: ResMut<GlobalComputeState>,
    mut staging_state: ResMut<GlobalStagingState>,
    sender: Res<TerrainChunkMeshSender>,
) {
    let gs = pool.grid_size;
    let vs = pool.voxel_size();
    let vc = gs;

    // ── 检测是否需要重建 ──
    // grid_min 对齐到 grid_world_size 世界坐标边界，
    // 只有 observer 跨越块边界才触发重建，保持 mesh 形状稳定。
    let (observer_pos, observer_force) = observer
        .as_deref()
        .map(|o: &TerrainObserver| (o.position, o.force_rebuild))
        .unwrap_or((Vec3::ZERO, 0));
    let grid_world_size = gs as f32 * vs;
    let grid_min = (observer_pos / grid_world_size).floor() * grid_world_size;

    let force_changed = observer_force != state.last_force_rebuild;
    let need_rebuild = state.pass == 0
        && (grid_min != state.last_grid_min || !state.rebuild_triggered || force_changed);

    // ── 阶段 0: dispatch compute passes ──
    if need_rebuild {
        // 检查所有管线是否已编译完成（异步编译，首帧可能未就绪）
        let pids = [
            pipeline.pass0,
            pipeline.pass1,
            pipeline.pass2,
            pipeline.pass3,
            pipeline.pass4,
            pipeline.pass5,
        ];
        let all_ready = pids
            .iter()
            .all(|pid| pipeline_cache.get_compute_pipeline(*pid).is_some());

        if !all_ready {
            // 管线尚未编译完成，等待下帧重试
            info!("Global DC: waiting for pipelines to compile...");
            return;
        }

        let uniform = GlobalUniformsGpu::new(grid_min, vs, gs);
        queue.write_buffer(&pipeline.uniform_buffer, 0, bytemuck::bytes_of(&uniform));

        state.last_grid_min = grid_min;
        state.grid_min = grid_min;
        state.rebuild_triggered = true;
        state.last_force_rebuild = observer_force;

        let encoder = render_context.command_encoder();

        // 清所有 mutable buffer（防 stale data 导致闪烁）
        encoder.clear_buffer(&pool.cross, 0, None);
        encoder.clear_buffer(&pool.counters, 0, None);
        encoder.clear_buffer(&pool.vertices, 0, None);
        encoder.clear_buffer(&pool.voxel_alloc, 0, None);
        encoder.clear_buffer(&pool.indices, 0, None);

        // dispatch 5 pass 在同一 encoder 内（GPU 保证顺序执行）
        let dispatch = |encoder: &mut bevy::render::render_resource::CommandEncoder,
                        pid: CachedComputePipelineId,
                        wg: (u32, u32, u32)| {
            // 此时管线必定就绪（已验证）
            let cp = pipeline_cache
                .get_compute_pipeline(pid)
                .expect("pipeline ready");
            let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
            cpass.set_pipeline(cp);
            cpass.set_bind_group(0, &pipeline.bind_group, &[]);
            cpass.dispatch_workgroups(wg.0, wg.1, wg.2);
        };

        let n = gs + 1;
        let wg0 = (n.div_ceil(8), n.div_ceil(8), n.div_ceil(8));
        dispatch(encoder, pipeline.pass0, wg0);

        let wg = (gs.div_ceil(8), gs.div_ceil(8), gs.div_ceil(8));
        dispatch(encoder, pipeline.pass1, wg);
        dispatch(encoder, pipeline.pass2, wg);
        dispatch(encoder, pipeline.pass3, wg);
        dispatch(encoder, pipeline.pass4, wg);
        dispatch(encoder, pipeline.pass5, (1, 1, 1));
        state.has_valid_data = true;
        state.pass = 1;
        info!("Global DC: rebuild at grid_min={grid_min:?} observer={observer_pos:?}");
    }
    // ── 阶段 1: 等 GPU → staging copy (仅当 readback_enabled) ──
    else if state.pass == 1 {
        if state.readback_enabled {
            let encoder = render_context.command_encoder();

            let vertex_cap =
                vc as u64 * vc as u64 * vc as u64 * size_of::<TerrainChunkVertex>() as u64;
            let index_cap = vc as u64 * vc as u64 * vc as u64 * 72 * 4;
            let voxel_alloc_size = vc as u64 * vc as u64 * vc as u64 * 4;

            let mk_staging = |label: &str, size: u64| {
                device.create_buffer(&BufferDescriptor {
                    label: Some(label),
                    size,
                    usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                })
            };

            let sv = mk_staging("global_staging_v", vertex_cap);
            let si = mk_staging("global_staging_i", index_cap);
            let sc = mk_staging("global_staging_c", 16);
            let sva = mk_staging("global_staging_va", voxel_alloc_size);

            encoder.copy_buffer_to_buffer(&pool.vertices, 0, &sv, 0, vertex_cap);
            encoder.copy_buffer_to_buffer(&pool.indices, 0, &si, 0, index_cap);
            encoder.copy_buffer_to_buffer(&pool.counters, 0, &sc, 0, 16);
            encoder.copy_buffer_to_buffer(&pool.voxel_alloc, 0, &sva, 0, voxel_alloc_size);

            staging_state.staging = Some(GlobalStaging {
                vertices: sv,
                indices: si,
                counters: sc,
                voxel_alloc: sva,
                vertex_cap,
                index_cap,
                voxel_alloc_size,
                mapped: Arc::new(AtomicBool::new(false)),
                map_started: false,
                grid_min: state.grid_min,
            });

            state.pass = 2;
        } else {
            // GPU indirect mode: skip staging + readback, go straight back to idle
            state.pass = 0;
        }
    }
    // ── 阶段 2: 等 staging copy → readback → mesh ──
    else if state.pass == 2 {
        do_readback(&device, &mut staging_state, &mut state, &sender, vs);
    }
}

/// 执行 staging readback：映射 counter → vertex → index buffer，
/// 构建 Mesh 并发送到主世界。
fn do_readback(
    device: &RenderDevice,
    staging_state: &mut GlobalStagingState,
    state: &mut GlobalComputeState,
    sender: &TerrainChunkMeshSender,
    vs: f32,
) {
    let Some(ref mut s) = staging_state.staging else {
        state.pass = 0;
        return;
    };

    // staging copy 本帧刚提交，等下一帧 GPU 执行
    if !s.map_started {
        s.map_started = true;
        return;
    }

    let wgpu_device = device.wgpu_device();

    // 尝试映射 counter buffer
    if !s.mapped.load(std::sync::atomic::Ordering::Acquire) {
        let flag = s.mapped.clone();
        s.counters
            .slice(..)
            .map_async(MapMode::Read, move |result| {
                if result.is_ok() {
                    flag.store(true, std::sync::atomic::Ordering::Release);
                }
            });
        let _ = wgpu_device.poll(bevy::render::render_resource::PollType::Poll);
        if !s.mapped.load(std::sync::atomic::Ordering::Acquire) {
            return; // 下帧继续等
        }
    }

    // 读 counters
    let counter_view = s.counters.slice(..).get_mapped_range();
    let vertex_count = u32::from_le_bytes(counter_view[0..4].try_into().expect("counter read"));
    let index_count = u32::from_le_bytes(counter_view[4..8].try_into().expect("counter read"));
    drop(counter_view);
    s.counters.unmap();

    // indirect draw command 已由 GPU pass 5 填充到 pool.indirect，
    // 内容与 counters[1] (index_count) 一致，无需单独读回。

    info!(
        "Global DC readback: {vertex_count} vertices, {index_count} indices ({} tris)",
        index_count.saturating_div(3)
    );

    if vertex_count == 0 || index_count == 0 {
        staging_state.staging = None;
        state.pass = 0;
        return;
    }

    // 映射顶点
    let v_flag = Arc::new(AtomicBool::new(false));
    {
        let vf = v_flag.clone();
        s.vertices
            .slice(..)
            .map_async(MapMode::Read, move |result| {
                if result.is_ok() {
                    vf.store(true, std::sync::atomic::Ordering::Release);
                }
            });
    }
    let _ = wgpu_device.poll(bevy::render::render_resource::PollType::Poll);
    if !v_flag.load(std::sync::atomic::Ordering::Acquire) {
        warn!("Global DC: vertex map timed out");
        return;
    }

    // 映射索引
    let i_flag = Arc::new(AtomicBool::new(false));
    {
        let inf = i_flag.clone();
        s.indices.slice(..).map_async(MapMode::Read, move |result| {
            if result.is_ok() {
                inf.store(true, std::sync::atomic::Ordering::Release);
            }
        });
    }
    let _ = wgpu_device.poll(bevy::render::render_resource::PollType::Poll);
    if !i_flag.load(std::sync::atomic::Ordering::Acquire) {
        warn!("Global DC: index map timed out");
        return;
    }

    let vertex_view = s.vertices.slice(..).get_mapped_range();
    let all_vertices: &[TerrainChunkVertex] =
        bytemuck::cast_slice(&vertex_view[..s.vertex_cap as usize]);
    let index_view = s.indices.slice(..).get_mapped_range();
    let all_indices: &[u32] = bytemuck::cast_slice(&index_view[..s.index_cap as usize]);

    // 读 voxel_alloc 用于 fixed slot → compact index remap
    let va_flag = Arc::new(AtomicBool::new(false));
    {
        let vaf = va_flag.clone();
        s.voxel_alloc
            .slice(..)
            .map_async(MapMode::Read, move |result| {
                if result.is_ok() {
                    vaf.store(true, std::sync::atomic::Ordering::Release);
                }
            });
    }
    let _ = wgpu_device.poll(bevy::render::render_resource::PollType::Poll);
    let voxel_alloc_data: Option<Vec<u32>> = if va_flag.load(std::sync::atomic::Ordering::Acquire) {
        let va_view = s.voxel_alloc.slice(..).get_mapped_range();
        let data = bytemuck::cast_slice(&va_view[..s.voxel_alloc_size as usize]).to_vec();
        drop(va_view);
        s.voxel_alloc.unmap();
        Some(data)
    } else {
        warn!("Global DC: voxel_alloc map timed out");
        None
    };

    let mesh = build_global_mesh(
        all_vertices,
        all_indices,
        vertex_count as usize,
        index_count as usize,
        voxel_alloc_data.as_deref(),
        s.grid_min,
        vs,
    );

    drop(vertex_view);
    drop(index_view);
    s.vertices.unmap();
    s.indices.unmap();

    if let Some(mesh) = mesh {
        let _ = sender.send(TerrainChunkMeshData {
            mesh,
            translation: s.grid_min,
        });
        info!("Global DC: mesh sent to main world");
    }

    staging_state.staging = None;
    state.pass = 0;
    // 保持 rebuild_triggered = true，后续只在 observer 移动时触发
}

/// 从 GPU compute 输出构建 Mesh。
///
/// 两条路径:
/// - voxel_alloc 为 Some → compact path: 顶点已在紧凑位置，索引已为 compact_index，无需 remap
/// - voxel_alloc 为 None → fallback: Phase 2 fixed slot 扫描 + remap 表
///
/// index buffer 布局: 每 voxel 72 u32 (12 edge × 6)，offset = grid_idx * 72。
/// vertex buffer: 每 voxel 1 slot (32 bytes)，offset = grid_idx。
fn build_global_mesh(
    all_vertices: &[TerrainChunkVertex],
    all_indices: &[u32],
    vertex_count: usize,
    index_count: usize,
    voxel_alloc: Option<&[u32]>,
    grid_min: Vec3,
    voxel_size: f32,
) -> Option<Mesh> {
    let gs = (all_vertices.len() as f64).cbrt() as u32;
    let total_slots = (gs as usize).pow(3).min(all_vertices.len());
    let grid_max = grid_min + Vec3::splat((gs + 1) as f32 * voxel_size);

    // 两条路径产生的数据
    let (positions, normals, tri_indices): (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<u32>);

    if voxel_alloc.is_some() {
        // ── Compact path: shader 已 scatter-write 到紧凑位置 ──
        // vertices[0..V] 是有效顶点，indices 已写入 compact_index 值
        let mut pos: Vec<[f32; 3]> = Vec::new();
        let mut nrm: Vec<[f32; 3]> = Vec::new();
        let mut clamped = 0u32;

        for v in all_vertices.iter().take(vertex_count) {
            let len = (v.position[0] * v.position[0]
                + v.position[1] * v.position[1]
                + v.position[2] * v.position[2])
                .sqrt();
            if len > 0.0001 {
                let p = [
                    v.position[0].clamp(grid_min.x, grid_max.x),
                    v.position[1].clamp(grid_min.y, grid_max.y),
                    v.position[2].clamp(grid_min.z, grid_max.z),
                ];
                if p != v.position {
                    clamped += 1;
                }
                pos.push(p);
                nrm.push(v.normal);
            }
        }

        if clamped > 0 {
            info!(
                "  QEF clamp: {clamped}/{} vertices clamped to grid",
                pos.len()
            );
        }
        info!(
            "  compact scan: {} valid / {} total",
            pos.len(),
            total_slots
        );

        if pos.is_empty() {
            return None;
        }

        // 无需 remap：index buffer 已包含 compact_index 值，直接当三角索引
        let vertex_count = pos.len() as u32;
        let mut idx: Vec<u32> = Vec::new();
        for &i in &all_indices[0..index_count] {
            if i < vertex_count {
                idx.push(i);
            }
        }
        if idx.is_empty() {
            return None;
        }

        positions = pos;
        normals = nrm;
        tri_indices = idx;
    } else {
        // ── Fallback: fixed slot scan with remap (Phase 2 风格) ──
        let mut remap: Vec<Option<u32>> = vec![None; total_slots];
        let mut pos: Vec<[f32; 3]> = Vec::new();
        let mut nrm: Vec<[f32; 3]> = Vec::new();
        let mut clamped = 0u32;

        for i in 0..total_slots {
            let v = &all_vertices[i];
            let len = (v.position[0] * v.position[0]
                + v.position[1] * v.position[1]
                + v.position[2] * v.position[2])
                .sqrt();
            if len > 0.0001 {
                let p = [
                    v.position[0].clamp(grid_min.x, grid_max.x),
                    v.position[1].clamp(grid_min.y, grid_max.y),
                    v.position[2].clamp(grid_min.z, grid_max.z),
                ];
                if p != v.position {
                    clamped += 1;
                }
                remap[i] = Some(pos.len() as u32);
                pos.push(p);
                nrm.push(v.normal);
            }
        }

        if clamped > 0 {
            info!(
                "  QEF clamp: {clamped}/{} vertices clamped to grid",
                pos.len()
            );
        }
        info!(
            "  fixed slot scan: {} valid / {} total",
            pos.len(),
            total_slots
        );

        if pos.is_empty() {
            return None;
        }

        // Phase 2: inner voxels [0, vc)³ → index = jx + jy*vc + jz*vc*vc
        // index buffer offset = inner_index * 72
        let vc = gs - 2;
        let mut idx: Vec<u32> = Vec::new();
        for jz in 0..vc {
            for jy in 0..vc {
                for jx in 0..vc {
                    let inner_idx = (jx + jy * vc + jz * vc * vc) as usize;
                    let base = inner_idx * 72;
                    if base + 71 >= all_indices.len() {
                        break;
                    }

                    for slot in 0..12 {
                        let off = base + slot * 6;
                        let s0 = all_indices[off] as usize;
                        let s1 = all_indices[off + 1] as usize;
                        let s2 = all_indices[off + 2] as usize;
                        let s3 = all_indices[off + 3] as usize;
                        let s4 = all_indices[off + 4] as usize;
                        let s5 = all_indices[off + 5] as usize;

                        let r0 = remap.get(s0).copied().flatten();
                        let r1 = remap.get(s1).copied().flatten();
                        let r2 = remap.get(s2).copied().flatten();
                        let r3 = remap.get(s3).copied().flatten();
                        let r4 = remap.get(s4).copied().flatten();
                        let r5 = remap.get(s5).copied().flatten();

                        if let (Some(r0), Some(r1), Some(r2), Some(r3), Some(r4), Some(r5)) =
                            (r0, r1, r2, r3, r4, r5)
                        {
                            if r0 != r1 && r1 != r2 && r0 != r2 {
                                idx.extend_from_slice(&[r0, r1, r2]);
                            }
                            if r3 != r4 && r4 != r5 && r3 != r5 {
                                idx.extend_from_slice(&[r3, r4, r5]);
                            }
                        }
                    }
                }
            }
        }

        if idx.is_empty() {
            return None;
        }

        positions = pos;
        normals = nrm;
        tri_indices = idx;
    }

    // ── Common: bbox + mesh 构建 ──
    let mut bmin = Vec3::splat(f32::MAX);
    let mut bmax = Vec3::splat(f32::MIN);
    for p in &positions {
        let v = Vec3::from_array(*p);
        bmin = bmin.min(v);
        bmax = bmax.max(v);
    }
    info!(
        "  global mesh: {} verts {} tris bbox=({:.1},{:.1},{:.1})→({:.1},{:.1},{:.1})",
        positions.len(),
        tri_indices.len() / 3,
        bmin.x,
        bmin.y,
        bmin.z,
        bmax.x,
        bmax.y,
        bmax.z,
    );

    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(bevy::mesh::Indices::U32(tri_indices));
    Some(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-4,
            "expected {a} ≈ {b} but diff = {}",
            (a - b).abs()
        );
    }

    fn v(position: [f32; 3], normal: [f32; 3]) -> TerrainChunkVertex {
        TerrainChunkVertex {
            position,
            normal,
            ..Default::default()
        }
    }

    /// 构造 5³ 的 vertex 槽位 buffer，并在给定 slots 放置有效顶点。
    fn vertex_buf_gs5(valid: &[(usize, [f32; 3])]) -> Vec<TerrainChunkVertex> {
        let mut buf = vec![TerrainChunkVertex::default(); 125];
        for &(i, pos) in valid {
            buf[i] = v(pos, [0.0, 1.0, 0.0]);
        }
        buf
    }

    fn mesh_positions(mesh: &Mesh) -> Vec<[f32; 3]> {
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("positions attr")
            .as_float3()
            .expect("float3")
            .to_vec()
    }

    fn mesh_normals(mesh: &Mesh) -> Vec<[f32; 3]> {
        mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
            .expect("normal attr")
            .as_float3()
            .expect("float3")
            .to_vec()
    }

    fn mesh_indices(mesh: &Mesh) -> Vec<u32> {
        match mesh.indices().expect("indices") {
            bevy::mesh::Indices::U32(v) => v.clone(),
            bevy::mesh::Indices::U16(v) => v.iter().map(|&x| x as u32).collect(),
        }
    }

    // ── GlobalUniformsGpu ──

    #[test]
    fn global_uniforms_new_packs_fields() {
        let u = GlobalUniformsGpu::new(Vec3::new(10.0, -20.0, 30.0), 0.5, 50);
        assert_eq!(u.grid_min, [10.0, -20.0, 30.0]);
        assert_eq!(u.pad0, 0);
        assert_approx(u.voxel_size, 0.5);
        assert_eq!(u.grid_size, 50);
        assert_eq!(u.pad1, [0; 2]);
        assert_eq!(u.neighbor_mask, 0);
        assert_eq!(u.pad_neighbor, 0);
        assert_eq!(u.seed, 42);
        assert_eq!(u.pad2, 0);
    }

    #[test]
    fn global_uniforms_layout_48_bytes() {
        assert_eq!(std::mem::size_of::<GlobalUniformsGpu>(), 48);
        let u = GlobalUniformsGpu::new(Vec3::new(1.0, 2.0, 3.0), 0.25, 64);
        let bytes = bytemuck::bytes_of(&u);
        // grid_min: 3×f32 @ 0
        assert_eq!(f32::from_le_bytes(bytes[0..4].try_into().expect("b0")), 1.0);
        assert_eq!(f32::from_le_bytes(bytes[4..8].try_into().expect("b4")), 2.0);
        assert_eq!(
            f32::from_le_bytes(bytes[8..12].try_into().expect("b8")),
            3.0
        );
        // pad0 @ 12 → 0
        assert_eq!(
            u32::from_le_bytes(bytes[12..16].try_into().expect("b12")),
            0
        );
        // voxel_size @ 16, grid_size @ 20
        assert_eq!(
            f32::from_le_bytes(bytes[16..20].try_into().expect("b16")),
            0.25
        );
        assert_eq!(
            u32::from_le_bytes(bytes[20..24].try_into().expect("b20")),
            64
        );
        // seed @ 40
        assert_eq!(
            u32::from_le_bytes(bytes[40..44].try_into().expect("b40")),
            42
        );
    }

    // ── build_global_mesh: compact path (voxel_alloc = Some) ──

    #[test]
    fn compact_path_builds_valid_mesh() {
        let verts = vertex_buf_gs5(&[
            (0, [1.0, 0.0, 0.0]),
            (1, [1.0, 1.0, 0.0]),
            (2, [2.0, 0.0, 0.0]),
            (3, [2.0, 1.0, 0.0]),
        ]);
        let indices = [0u32, 1, 2, 0, 2, 3];
        let mesh = build_global_mesh(&verts, &indices, 125, 6, Some(&[]), Vec3::ZERO, 1.0)
            .expect("compact path should produce a mesh");
        assert_eq!(mesh.count_vertices(), 4);
        assert_eq!(mesh_indices(&mesh), indices);
        assert_eq!(mesh_positions(&mesh)[0], [1.0, 0.0, 0.0]);
        assert_eq!(mesh_normals(&mesh)[0], [0.0, 1.0, 0.0]);
    }

    #[test]
    fn compact_path_clamps_out_of_bounds_vertices() {
        // grid_max = (5+1)*1.0 = 6；顶点超出 [0,6] 立方体被 clamp
        let verts = vertex_buf_gs5(&[(0, [50.0, -3.0, 0.0]), (1, [1.0, 1.0, 0.0])]);
        let indices = [0u32, 1, 0];
        let mesh = build_global_mesh(&verts, &indices, 125, 3, Some(&[]), Vec3::ZERO, 1.0)
            .expect("mesh with clamped vertex");
        let pos = mesh_positions(&mesh);
        assert!(
            pos.contains(&[6.0, 0.0, 0.0]),
            "out-of-bounds vertex clamped: {pos:?}"
        );
    }

    #[test]
    fn compact_path_no_valid_vertices_returns_none() {
        let verts = vec![TerrainChunkVertex::default(); 125];
        assert!(build_global_mesh(&verts, &[], 125, 0, Some(&[]), Vec3::ZERO, 1.0).is_none());
    }

    #[test]
    fn compact_path_all_indices_invalid_returns_none() {
        let verts = vertex_buf_gs5(&[(0, [1.0, 0.0, 0.0])]);
        // 索引全部 >= 有效顶点数(1) → 被过滤 → idx 空 → None
        let indices = [10u32, 11, 12, 13, 14, 15];
        assert!(build_global_mesh(&verts, &indices, 125, 6, Some(&[]), Vec3::ZERO, 1.0).is_none());
    }

    #[test]
    fn compact_path_filters_invalid_indices() {
        let verts = vertex_buf_gs5(&[
            (0, [1.0, 0.0, 0.0]),
            (1, [1.0, 1.0, 0.0]),
            (2, [2.0, 0.0, 0.0]),
        ]);
        let indices = [0u32, 1, 2, 99, 98, 97];
        let mesh = build_global_mesh(&verts, &indices, 125, 6, Some(&[]), Vec3::ZERO, 1.0)
            .expect("valid prefix survives");
        assert_eq!(mesh.count_vertices(), 3);
        assert_eq!(mesh_indices(&mesh), vec![0, 1, 2]);
    }

    #[test]
    fn compact_path_vertex_count_limits_scan() {
        // vertex_count=2 → 只扫描前 2 个槽位，第 3 个顶点被忽略
        let verts = vertex_buf_gs5(&[
            (0, [1.0, 0.0, 0.0]),
            (1, [1.0, 1.0, 0.0]),
            (2, [2.0, 0.0, 0.0]),
        ]);
        let indices = [0u32, 1, 2];
        let mesh = build_global_mesh(&verts, &indices, 2, 3, Some(&[]), Vec3::ZERO, 1.0)
            .expect("mesh from first two slots");
        assert_eq!(mesh.count_vertices(), 2);
    }

    // ── build_global_mesh: fallback path (voxel_alloc = None) ──

    fn fallback_index_buf() -> Vec<u32> {
        vec![0; 27 * 72] // gs=5 → vc=3 → 27 inner voxels × 72
    }

    #[test]
    fn fallback_path_builds_valid_mesh() {
        let verts = vertex_buf_gs5(&[
            (0, [1.0, 0.0, 0.0]),
            (1, [1.0, 1.0, 0.0]),
            (2, [2.0, 0.0, 0.0]),
        ]);
        let mut indices = fallback_index_buf();
        // inner voxel (0,0,0) → inner_idx 0 → base 0；两个非退化三角形
        indices[0..6].copy_from_slice(&[0, 1, 2, 0, 2, 1]);
        let mesh = build_global_mesh(&verts, &indices, 125, 0, None, Vec3::ZERO, 1.0)
            .expect("fallback path should produce a mesh");
        assert_eq!(mesh.count_vertices(), 3);
        assert_eq!(mesh_indices(&mesh), vec![0, 1, 2, 0, 2, 1]);
    }

    #[test]
    fn fallback_path_scans_all_remap_slots() {
        // 顶点放在高位槽位（非 voxel0 引用的常规位置），remap 表扫描所有 total_slots
        let verts = vertex_buf_gs5(&[
            (100, [1.0, 0.0, 0.0]),
            (101, [1.0, 1.0, 0.0]),
            (102, [2.0, 0.0, 0.0]),
        ]);
        let mut indices = fallback_index_buf();
        indices[0..6].copy_from_slice(&[100, 101, 102, 100, 102, 101]);
        let mesh = build_global_mesh(&verts, &indices, 125, 0, None, Vec3::ZERO, 1.0)
            .expect("high slots are remapped");
        assert_eq!(mesh.count_vertices(), 3);
        assert_eq!(mesh_indices(&mesh), vec![0, 1, 2, 0, 2, 1]);
    }

    #[test]
    fn fallback_path_no_valid_vertices_returns_none() {
        let verts = vec![TerrainChunkVertex::default(); 125];
        assert!(
            build_global_mesh(&verts, &fallback_index_buf(), 0, 0, None, Vec3::ZERO, 1.0).is_none()
        );
    }

    #[test]
    fn fallback_path_degenerate_triangles_skipped() {
        // 只有一个有效顶点 → 所有三角形退化 → idx 空 → None
        let verts = vertex_buf_gs5(&[(0, [1.0, 0.0, 0.0])]);
        let mut indices = fallback_index_buf();
        indices[0..6].copy_from_slice(&[0, 0, 0, 0, 0, 0]);
        assert!(build_global_mesh(&verts, &indices, 125, 0, None, Vec3::ZERO, 1.0).is_none());
    }

    #[test]
    fn fallback_path_partial_degenerate_keeps_valid() {
        let verts = vertex_buf_gs5(&[
            (0, [1.0, 0.0, 0.0]),
            (1, [1.0, 1.0, 0.0]),
            (2, [2.0, 0.0, 0.0]),
        ]);
        let mut indices = fallback_index_buf();
        // 第一个三角形非退化，第二个三角形 (0,0,1) 退化 → 只保留第一个
        indices[0..6].copy_from_slice(&[0, 1, 2, 0, 0, 1]);
        let mesh = build_global_mesh(&verts, &indices, 125, 0, None, Vec3::ZERO, 1.0)
            .expect("valid triangle kept");
        assert_eq!(mesh_indices(&mesh), vec![0, 1, 2]);
    }

    #[test]
    fn fallback_path_short_index_buffer_breaks_early() {
        let verts = vertex_buf_gs5(&[
            (0, [1.0, 0.0, 0.0]),
            (1, [1.0, 1.0, 0.0]),
            (2, [2.0, 0.0, 0.0]),
        ]);
        // 索引 buffer 不够 72 个 → 直接 break → 无三角形 → None
        let short = vec![0u32; 40];
        assert!(build_global_mesh(&verts, &short, 125, 0, None, Vec3::ZERO, 1.0).is_none());
    }

    #[test]
    fn fallback_path_clamps_vertices() {
        let verts = vertex_buf_gs5(&[(0, [50.0, -3.0, 0.0])]);
        let mut indices = fallback_index_buf();
        indices[0..6].copy_from_slice(&[0, 0, 0, 0, 0, 0]);
        // 即使三角形退化，也先验证 clamp 路径执行（顶点被 clamp 后 remap）
        let _ = build_global_mesh(&verts, &indices, 125, 0, None, Vec3::ZERO, 1.0);
    }

    #[test]
    fn fallback_path_small_grid_vc_1() {
        // gs=3 → vc=1 → 单个 inner voxel
        let mut verts = vec![TerrainChunkVertex::default(); 27];
        verts[0] = v([0.5, 0.0, 0.0], [0.0, 1.0, 0.0]);
        verts[1] = v([0.5, 0.5, 0.0], [0.0, 1.0, 0.0]);
        verts[2] = v([0.6, 0.0, 0.5], [0.0, 1.0, 0.0]);
        let mut indices = vec![0u32; 72];
        indices[0..6].copy_from_slice(&[0, 1, 2, 0, 2, 1]);
        let mesh = build_global_mesh(&verts, &indices, 27, 0, None, Vec3::ZERO, 0.5)
            .expect("single voxel mesh");
        assert_eq!(mesh.count_vertices(), 3);
        assert_eq!(mesh_indices(&mesh), vec![0, 1, 2, 0, 2, 1]);
    }
}
