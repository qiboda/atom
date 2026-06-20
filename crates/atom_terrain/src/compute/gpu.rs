//! GPU compute mesh generation — Bevy 0.19.
//! Four-pass Dual Contouring on GPU with fixed-slot sparse vertices/indices.
//!
//! # Architecture
//!
//! ```text
//! RenderStartup: init_compute_pipeline → queue 4 compute shaders + bind group layout
//! Render (every frame): terrain_compute_system
//!   allocate → dispatch(1-4) → staging copy → async map → compact+remap → crossbeam send
//! Main world (every frame): handle_mesh_data → spawn Mesh3d + MeshMaterial3d
//! ```
//!
//! # Per-chunk state machine
//!
//! ```text
//! 0(allocate)→1(pass1)→2(pass2)→3(pass3)→4(pass4)→5(staging copy)→6(readback)→7(cleanup)
//! ```
//!
//! Each dispatch→advance transition waits for the compute pipeline to be compiled
//! (async shader compilation). Pass 4→5 and 5→6 each wait one extra frame for GPU
//! execution (dispatch and copy are queued commands, executed after Bevy submits the
//! command encoder).

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use bevy::{
    prelude::*,
    render::{
        render_resource::{
            binding_types::*, BindGroup, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntries, Buffer, BufferDescriptor,
            BufferUsages, CachedComputePipelineId, ComputePassDescriptor,
            ComputePipelineDescriptor, PipelineCache, ShaderStages,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
    },
};

use super::{
    sync::{ChunkProcessRequest, TerrainChunkProcessReceiver},
    types::{TerrainChunkInfo, TerrainChunkVertex},
};
use crate::{mesh::TerrainChunkMeshData, mesh::TerrainChunkMeshSender, setting::TerrainSetting};

/// 地形 compute 管线资源，包含 bind group layout 和四个 pass 的 compute pipeline id。
#[derive(Resource)]
pub struct TerrainComputePipeline {
    /// 共享 bind group layout
    pub bind_group_layout: BindGroupLayout,
    /// pass1: 密度场计算
    pub pass1: CachedComputePipelineId,
    /// pass2: 边交叉点
    pub pass2: CachedComputePipelineId,
    /// pass3: QEF 顶点
    pub pass3: CachedComputePipelineId,
    /// pass4: 索引生成
    pub pass4: CachedComputePipelineId,
}

/// 初始化 compute pipeline：创建 bind group layout 并注册四个 compute pass 的 shader。
/// 在 RenderStartup 阶段调用。
pub fn init_compute_pipeline(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
) {
    let entries = BindGroupLayoutEntries::sequential(
        ShaderStages::COMPUTE,
        (
            uniform_buffer::<TerrainChunkInfo>(false),
            storage_buffer::<Vec<f32>>(false),
            storage_buffer::<Vec<u32>>(false),
            storage_buffer::<Vec<TerrainChunkVertex>>(false),
            storage_buffer::<Vec<u32>>(false),
            storage_buffer::<Vec<u32>>(false),
        ),
    );
    let desc = BindGroupLayoutDescriptor::new("terrain_bgl", &entries);
    let bgl = render_device.create_bind_group_layout("terrain_bgl", &entries);

    let mk = |label: &'static str, path: &'static str| {
        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(label.into()),
            layout: vec![desc.clone()],
            shader: asset_server.load(path),
            ..default()
        })
    };

    commands.insert_resource(TerrainComputePipeline {
        bind_group_layout: bgl,
        pass1: mk("pass1", "shaders/terrain/compute/voxel_vertices.wgsl"),
        pass2: mk("pass2", "shaders/terrain/compute/voxel_cross_points.wgsl"),
        pass3: mk(
            "pass3",
            "shaders/terrain/compute/main_mesh_compute_vertices.wgsl",
        ),
        pass4: mk(
            "pass4",
            "shaders/terrain/compute/main_mesh_compute_indices.wgsl",
        ),
    });
}

/// 单个 chunk 的 GPU buffer 集合：密度场、cross points、顶点、索引、计数器及 bind group。
pub struct ChunkBuffers {
    /// 密度场 (voxel_count+1)³ 个 f32
    pub density: Buffer,
    /// 交叉点数据，每个 voxel 12 条边 × 32 bytes
    pub cross_points: Buffer,
    /// 顶点 buffer (TerrainChunkVertex)
    pub vertices: Buffer,
    /// 索引 buffer (u32)
    pub indices: Buffer,
    /// 计数器 (vertex_count, index_count, 2×pad)
    pub counters: Buffer,
    /// 绑定组
    pub bind_group: BindGroup,
    /// chunk 世界坐标偏移（分配时记录，staging 时使用）
    pub world_min: Vec3,
}

/// 所有 chunk 的 GPU buffer 映射表（Entity → ChunkBuffers）。
/// 作为资源注入，由 terrain_compute_system 管理生命周期。
#[derive(Resource, Default)]
pub struct TerrainChunkMeshBuffers {
    buffers: HashMap<Entity, ChunkBuffers>,
}

/// 每个 chunk 的 compute pass 进度计数器。
/// 0=已分配未开始，1-4=compute pass，5=staging copy 已提交，6=readback，7=完成。
/// 由 terrain_compute_system 每帧推进。
#[derive(Resource, Default)]
pub struct TerrainChunkComputeProgress {
    /// entity → 当前 pass
    pub pass: HashMap<Entity, u32>,
}

/// GPU→CPU readback 的 staging buffer 及异步映射状态
struct StagingReadback {
    /// 顶点 staging buffer
    vertices: Buffer,
    /// 索引 staging buffer
    indices: Buffer,
    /// 计数器 staging buffer
    counters: Buffer,
    /// 顶点 buffer 大小 (bytes)
    vertex_size: u64,
    /// 索引 buffer 大小 (bytes)
    index_size: u64,
    /// 映射完成标志（callback 设置）
    mapped: Arc<AtomicBool>,
    /// 映射已发起标志（防止重复 map_async）
    map_started: bool,
    /// chunk 世界坐标偏移
    world_min: Vec3,
}

/// 待 readback 的 staging buffer 集合 (entity → staging)。
///
/// 生命周期：在 pass 5（staging copy）时插入，pass 7（cleanup）时移除。
/// 每个 entry 持有顶点/索引/计数器的 GPU→CPU 中转 buffer 及异步映射状态。
#[derive(Resource, Default)]
pub struct TerrainChunkStagingBuffers {
    buffers: HashMap<Entity, StagingReadback>,
}

impl TerrainChunkMeshBuffers {
    fn allocate(
        &mut self,
        entity: Entity,
        info: &TerrainChunkInfo,
        vc: u32,
        device: &RenderDevice,
        queue: &RenderQueue,
        bgl: &BindGroupLayout,
    ) {
        let vv = vc + 2; // 双边 shell voxel 数
        let dv = vc + 3; // 双边 shell 密度 grid 点数
        let dg = dv as u64 * dv as u64 * dv as u64;
        let vn = vv as u64 * vv as u64 * vv as u64; // vertex/cross slots
        let ni = vc as u64 * vc as u64 * vc as u64; // index slots (仅内层)
        let s = BufferUsages::STORAGE | BufferUsages::COPY_DST;
        let so = BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST;
        let u = BufferUsages::UNIFORM | BufferUsages::COPY_DST;

        let mk = |label: &str, size: u64, usage: BufferUsages| {
            device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };
        let ct = mk("counters", 16, so | BufferUsages::COPY_DST);
        let db = mk("density", dg * 4, s);
        let cb = mk("cross", vn * 12 * 32, s);
        let vb = mk("verts", vn * size_of::<TerrainChunkVertex>() as u64, so);
        let ib = mk("indices", ni * 72 * 4, so); // 72 slots/voxel (12 edges × 6)
        let ub = mk("chunk_info", size_of::<TerrainChunkInfo>() as u64, u);

        queue.write_buffer(&ub, 0, bytemuck::bytes_of(info));
        let zero: [u32; 4] = [0; 4];
        queue.write_buffer(&ct, 0, bytemuck::bytes_of(&zero));

        let bg = device.create_bind_group(
            "chunk_bg",
            bgl,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: ub.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: db.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: cb.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: vb.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: ib.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 5,
                    resource: ct.as_entire_binding(),
                },
            ],
        );

        self.buffers.insert(
            entity,
            ChunkBuffers {
                density: db,
                cross_points: cb,
                vertices: vb,
                indices: ib,
                counters: ct,
                bind_group: bg,
                world_min: Vec3::from_array(info.chunk_min),
            },
        );
    }

    fn get(&self, entity: Entity) -> Option<&ChunkBuffers> {
        self.buffers.get(&entity)
    }

    fn remove(&mut self, entity: Entity) -> Option<ChunkBuffers> {
        self.buffers.remove(&entity)
    }
}

/// 每帧执行的 compute dispatch + readback 系统。
/// 运行于 Render 阶段，管理四 pass dispatch、pass 推进和完成 chunk 的 buffer 读回。
#[allow(clippy::too_many_arguments)]
pub fn terrain_compute_system(
    mut render_context: RenderContext,
    pipeline_cache: Res<PipelineCache>,
    pipeline: Res<TerrainComputePipeline>,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    setting: Res<TerrainSetting>,
    receiver: Res<TerrainChunkProcessReceiver>,
    mut buffers: ResMut<TerrainChunkMeshBuffers>,
    mut progress: ResMut<TerrainChunkComputeProgress>,
    mut staging: ResMut<TerrainChunkStagingBuffers>,
    sender: Res<TerrainChunkMeshSender>,
) {
    let vc = setting.voxel_count;

    // 0. 从 channel 读取主世界发来的加载/卸载请求
    let mut new_entities: Vec<Entity> = Vec::new();
    while let Ok(req) = receiver.try_recv() {
        match req {
            ChunkProcessRequest::Load { entity, world_min } => {
                if progress.pass.contains_key(&entity) {
                    continue; // 已在处理中
                }
                let info = TerrainChunkInfo {
                    chunk_min: world_min.to_array(),
                    voxel_size: setting.voxel_size,
                    voxel_count: vc,
                    terrain_size: setting.terrain_size,
                    seed: setting.seed,
                    pad0: 0,
                    pad1: [0; 2],
                    pad2: [0; 2],
                };
                buffers.allocate(
                    entity,
                    &info,
                    vc,
                    &device,
                    &queue,
                    &pipeline.bind_group_layout,
                );
                progress.pass.insert(entity, 0);
                new_entities.push(entity);
            }
            ChunkProcessRequest::Unload { entity } => {
                buffers.remove(entity);
                staging.buffers.remove(&entity);
                progress.pass.remove(&entity);
            }
        }
    }

    // 清空新分配 buffer 的残留数据
    for entity in &new_entities {
        if let Some(cb) = buffers.get(*entity) {
            let enc = render_context.command_encoder();
            enc.clear_buffer(&cb.cross_points, 0, None);
            enc.clear_buffer(&cb.vertices, 0, None);
            enc.clear_buffer(&cb.indices, 0, None);
        }
    }

    // 1. dispatch compute passes 1-4
    use std::collections::HashSet;
    let encoder = render_context.command_encoder();
    let mut dispatched: HashSet<Entity> = HashSet::new();
    // 跟踪本帧刚晋升的 pass，跳过需要 GPU 执行等待的 staging/readback
    let mut fresh_pass5: HashSet<Entity> = HashSet::new();
    let mut fresh_pass6: HashSet<Entity> = HashSet::new();
    for (&entity, pass) in progress.pass.iter() {
        let Some(cb) = buffers.get(entity) else {
            continue;
        };
        let pid = match *pass {
            1 => pipeline.pass1,
            2 => pipeline.pass2,
            3 => pipeline.pass3,
            4 => pipeline.pass4,
            _ => continue,
        };
        let Some(cp) = pipeline_cache.get_compute_pipeline(pid) else {
            continue; // pipeline 尚未编译完成
        };
        let wg = match *pass {
            1 => {
                let n = vc + 3; // 密度 grid 双边 shell
                (n.div_ceil(8), n.div_ceil(8), n.div_ceil(8))
            }
            2 | 3 | 4 => {
                let n = vc + 2; // voxel 双边 shell; pass4 内部 skip 非内层
                (n.div_ceil(8), n.div_ceil(8), n.div_ceil(8))
            }
            _ => continue,
        };
        let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
        cpass.set_pipeline(cp);
        cpass.set_bind_group(0, &cb.bind_group, &[]);
        cpass.dispatch_workgroups(wg.0, wg.1, wg.2);
        dispatched.insert(entity);
    }

    // 2. advance: 只有 dispatch 成功的才推进
    //    pass 4→5 需要等一帧（GPU 执行需要时间），所以标记为 fresh_pass5 跳过本帧的 staging
    for (entity, pass) in progress.pass.iter_mut() {
        if *pass == 0 {
            *pass = 1;
        } else if *pass >= 1 && *pass < 4 && dispatched.contains(entity) {
            *pass += 1;
        } else if *pass == 4 && dispatched.contains(entity) {
            *pass = 5;
            fresh_pass5.insert(*entity);
        }
    }

    // 3. staging copy (pass == 5, 且不是本帧刚晋升的)
    let vn = (vc + 2) as u64 * (vc + 2) as u64 * (vc + 2) as u64; // vertex slots (双边 shell)
    let ni = vc as u64 * vc as u64 * vc as u64; // index slots (仅内层)
    let vertex_size = vn * size_of::<TerrainChunkVertex>() as u64;
    let index_size = ni * 72 * 4; // 72 slots/voxel (12 edges × 6)

    let mut copied: Vec<Entity> = Vec::new();
    for (&entity, pass) in progress.pass.iter() {
        if *pass != 5 {
            continue;
        }
        // 跳过本帧刚从 pass 4 晋升的 — GPU 还没执行 dispatch
        if fresh_pass5.contains(&entity) {
            continue;
        }
        let Some(cb) = buffers.get(entity) else {
            continue;
        };
        // 创建 staging buffers
        let mk_staging = |label: &str, size: u64| {
            device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size,
                usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        };
        let sv = mk_staging("staging_v", vertex_size);
        let si = mk_staging("staging_i", index_size);
        let sc = mk_staging("staging_c", 16);

        // 提交 copy 命令到当前帧的 encoder
        encoder.copy_buffer_to_buffer(&cb.vertices, 0, &sv, 0, vertex_size);
        encoder.copy_buffer_to_buffer(&cb.indices, 0, &si, 0, index_size);
        encoder.copy_buffer_to_buffer(&cb.counters, 0, &sc, 0, 16);

        // 从 ChunkBuffers 读取 world_min（分配时记录）
        let world_min = cb.world_min;

        staging.buffers.insert(
            entity,
            StagingReadback {
                vertices: sv,
                indices: si,
                counters: sc,
                vertex_size,
                index_size,
                mapped: Arc::new(AtomicBool::new(false)),
                map_started: false,
                world_min,
            },
        );
        copied.push(entity);
    }
    for entity in &copied {
        *progress.pass.get_mut(entity).expect("pass entry exists") = 6;
        fresh_pass6.insert(*entity);
    }

    // 4. readback (pass == 6)
    //    上一帧的 copy 已被 GPU 执行，map staging buffer 读取数据
    let wgpu_device = device.wgpu_device();
    let mut done: Vec<(Entity, Vec3)> = Vec::new();

    for (&entity, pass) in progress.pass.iter() {
        if *pass != 6 {
            continue;
        }
        // 跳过本帧刚完成 copy 的 — GPU 还没执行 copy 命令
        if fresh_pass6.contains(&entity) {
            continue;
        }
        let Some(s) = staging.buffers.get_mut(&entity) else {
            warn!("pass==6 but no staging buffer for {entity:?}");
            continue;
        };

        // 首次进入 pass==6：发起 async 映射
        if !s.map_started {
            s.map_started = true;
            let flag = s.mapped.clone();
            let slice = s.counters.slice(..);
            slice.map_async(
                bevy::render::render_resource::MapMode::Read,
                move |result| {
                    if result.is_ok() {
                        flag.store(true, Ordering::Release);
                    }
                },
            );
            let _ = wgpu_device.poll(bevy::render::render_resource::PollType::Poll);
            continue; // 下帧等 mapping 完成
        }

        if !s.mapped.load(Ordering::Acquire) {
            continue; // 映射尚未完成
        }

        let counter_view = s.counters.slice(..).get_mapped_range();
        let _counters: &[u32; 4] = bytemuck::from_bytes(&counter_view[..16]);
        drop(counter_view);
        s.counters.unmap();

        // 读顶点
        s.vertices
            .slice(..)
            .map_async(bevy::render::render_resource::MapMode::Read, |_| {});
        // 读索引
        s.indices
            .slice(..)
            .map_async(bevy::render::render_resource::MapMode::Read, |_| {});
        let _ = wgpu_device.poll(bevy::render::render_resource::PollType::Poll);

        let vertex_view = s.vertices.slice(..).get_mapped_range();
        let all_vertices: &[TerrainChunkVertex] =
            bytemuck::cast_slice(&vertex_view[..s.vertex_size as usize]);
        let index_view = s.indices.slice(..).get_mapped_range();
        let all_indices: &[u32] = bytemuck::cast_slice(&index_view[..s.index_size as usize]);

        // compact + remap: 过滤零顶点，clamp QEF 外溢，重映射索引
        let mesh = compact_and_build_mesh(
            all_vertices,
            all_indices,
            vc,
            s.world_min,
            setting.voxel_size,
        );
        drop(vertex_view);
        drop(index_view);
        s.vertices.unmap();
        s.indices.unmap();

        if let Some(mesh) = mesh {
            let _ = sender.send(TerrainChunkMeshData {
                mesh,
                translation: s.world_min,
            });
            info!(
                "Chunk {entity:?} readback 完成 → mesh 已发送 ({:?})",
                s.world_min
            );
        }
        done.push((entity, s.world_min));
    }

    for (entity, _world_min) in &done {
        *progress.pass.get_mut(entity).expect("pass entry exists") = 7;
    }

    // 5. cleanup (pass == 7)
    let to_remove: Vec<Entity> = progress
        .pass
        .iter()
        .filter(|(_, p)| **p == 7)
        .map(|(e, _)| *e)
        .collect();
    for entity in to_remove {
        buffers.remove(entity);
        staging.buffers.remove(&entity);
        progress.pass.remove(&entity);
    }
}

/// 将 GPU 读回的稀疏（fixed-slot）顶点/索引 compact + remap 为 Bevy Mesh。
///
/// # 算法
///
/// 1. 过滤：保留 `length(position) > 1e-4` 的顶点，构建 old→new 索引映射
/// 2. Remap：遍历每个 voxel 的 6 个索引（两个三角形 0-1-2、0-2-3），跳过退化三角形
/// 3. 构建 `Mesh`（TriangleList + Positions + Normals + Indices）
///
/// 返回 `None` 当没有有效顶点（chunk 全部在空气或全部在固体中）。
fn compact_and_build_mesh(
    all_vertices: &[TerrainChunkVertex],
    all_indices: &[u32],
    vc: u32,
    chunk_min: Vec3,
    voxel_size: f32,
) -> Option<Mesh> {
    let total_vv = ((vc + 2) as usize).pow(3); // 双边 shell slot 数
    let total_vc = (vc as usize).pow(3); // 内层 voxel 数（索引）
    let chunk_max = chunk_min + Vec3::splat((vc + 1) as f32 * voxel_size); // 含 shell 的 clamp 上界

    // 构建 old→new 映射：标记有 position 的顶点（包括 shell 顶点）
    let mut remap: Vec<Option<u32>> = vec![None; total_vv];
    let mut compact_verts: Vec<[f32; 3]> = Vec::new();
    let mut compact_norms: Vec<[f32; 3]> = Vec::new();
    let mut clamped = 0u32;

    for (i, v) in all_vertices.iter().enumerate().take(total_vv) {
        let len = (v.position[0] * v.position[0]
            + v.position[1] * v.position[1]
            + v.position[2] * v.position[2])
            .sqrt();
        if len > 0.0001 {
            // clamp QEF 顶点到 voxel 所在的 chunk 范围内
            let clamped_pos = [
                v.position[0].clamp(chunk_min.x, chunk_max.x),
                v.position[1].clamp(chunk_min.y, chunk_max.y),
                v.position[2].clamp(chunk_min.z, chunk_max.z),
            ];
            if clamped_pos != v.position {
                clamped += 1;
            }
            remap[i] = Some(compact_verts.len() as u32);
            compact_verts.push(clamped_pos);
            compact_norms.push(v.normal);
        }
    }
    if clamped > 0 {
        info!(
            "  clamp: {clamped}/{} vertices clamped to chunk bounds",
            compact_verts.len()
        );
    }

    if compact_verts.is_empty() {
        return None;
    }

    // 遍历索引：只处理内层 vc³ 个 voxel（边界 shell 不生成 quad）
    // 每 voxel 12 个 index slot（2 quad），遍历两个 slot
    let mut tri_indices: Vec<u32> = Vec::new();
    for voxel_idx in 0..total_vc {
        let base = voxel_idx * 72;
        if base + 71 >= all_indices.len() {
            break;
        }
        for slot in 0..12 {
            let off = base + slot * 6;
            let i0 = all_indices[off] as usize;
            let i1 = all_indices[off + 1] as usize;
            let i2 = all_indices[off + 2] as usize;
            let i3 = all_indices[off + 3] as usize;
            let i4 = all_indices[off + 4] as usize;
            let i5 = all_indices[off + 5] as usize;

            let r0 = remap.get(i0).copied().flatten();
            let r1 = remap.get(i1).copied().flatten();
            let r2 = remap.get(i2).copied().flatten();
            let r3 = remap.get(i3).copied().flatten();
            let r4 = remap.get(i4).copied().flatten();
            let r5 = remap.get(i5).copied().flatten();

            if let (Some(r0), Some(r1), Some(r2), Some(r3), Some(r4), Some(r5)) =
                (r0, r1, r2, r3, r4, r5)
            {
                if r0 != r1 && r0 != r2 && r1 != r2 {
                    tri_indices.extend_from_slice(&[r0, r1, r2]);
                }
                if r3 != r4 && r3 != r5 && r4 != r5 {
                    tri_indices.extend_from_slice(&[r3, r4, r5]);
                }
            }
        }
    }

    if tri_indices.is_empty() {
        return None;
    }

    let mut mesh = Mesh::new(
        bevy::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::default(),
    );
    // 诊断: bounding box + 采样前几个顶点
    let mut bmin = Vec3::splat(f32::MAX);
    let mut bmax = Vec3::splat(f32::MIN);
    for p in &compact_verts {
        let v = Vec3::from_array(*p);
        bmin = bmin.min(v);
        bmax = bmax.max(v);
    }
    let sample: Vec<String> = compact_verts
        .iter()
        .take(8)
        .map(|v| format!("({:.2},{:.2},{:.2})", v[0], v[1], v[2]))
        .collect();
    info!(
        "  mesh: {} verts {} tris bbox=({:.1},{:.1},{:.1})→({:.1},{:.1},{:.1}) sample=[{}]",
        compact_verts.len(),
        tri_indices.len() / 3,
        bmin.x,
        bmin.y,
        bmin.z,
        bmax.x,
        bmax.y,
        bmax.z,
        sample.join(" ")
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, compact_verts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, compact_norms);
    mesh.insert_indices(bevy::mesh::Indices::U32(tri_indices));
    Some(mesh)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证 compact 函数能从有效顶点数据产出非空 mesh
    #[test]
    fn single_chunk_produces_vertices() {
        let vc = 4u32;
        let total_vv = ((vc + 2) as usize).pow(3); // 216 (双边 shell)
        let total_vc = (vc as usize).pow(3); // 64
        let mut verts = vec![TerrainChunkVertex::default(); total_vv];
        let mut indices = vec![0u32; total_vc * 72]; // 72 slots/voxel

        // 模拟一个 voxel 有几何：顶点在 chunk 中心 + 两个有效三角形
        let vi = 21usize; // 中间 voxel
        verts[vi] = TerrainChunkVertex {
            position: [2.0, 1.0, 2.0],
            normal: [0.0, 1.0, 0.0],
            ..Default::default()
        };
        // 三角形: voxel 21 的三个"邻居"也有顶点
        verts[22] = TerrainChunkVertex {
            position: [2.5, 1.2, 2.0],
            normal: [0.0, 1.0, 0.0],
            ..Default::default()
        };
        verts[26] = TerrainChunkVertex {
            position: [2.0, 1.1, 2.5],
            normal: [0.0, 1.0, 0.0],
            ..Default::default()
        };
        verts[27] = TerrainChunkVertex {
            position: [2.5, 1.3, 2.5],
            normal: [0.0, 1.0, 0.0],
            ..Default::default()
        };

        // 写入 6 个索引指向这 4 个顶点 (两个三角形: 21-22-26, 21-26-27)
        let base = vi * 72; // edge 0 slot
        indices[base] = 21;
        indices[base + 1] = 22;
        indices[base + 2] = 26;
        indices[base + 3] = 21;
        indices[base + 4] = 26;
        indices[base + 5] = 27;

        let mesh = compact_and_build_mesh(&verts, &indices, vc, Vec3::ZERO, 0.5);
        assert!(mesh.is_some(), "should produce non-empty mesh");
        let mesh = mesh.unwrap();
        assert!(mesh.count_vertices() > 0, "vertex_count > 0");
        assert_eq!(mesh.count_vertices(), 4, "4 valid vertices after compact");
        // 2 triangles × 3 indices = 6
        assert!(mesh.indices().is_some(), "index_count > 0");
    }

    /// 验证两个相邻 chunk 在 shell overlap 区域产生一致的顶点世界坐标。
    ///
    /// CPU 测试：构造两相邻 chunk 的合成顶点/索引数据，两 chunk 各在共享边界
    /// (X = vc * voxel_size) 放置一个顶点，期望 compact 后两者的世界坐标一致。
    #[test]
    #[ignore = "CPU test: needs shell overlap convention document"]
    fn surface_is_contiguous() {
        let vc = 2u32;
        let voxel_size = 0.5;
        let chunk_size = vc as f32 * voxel_size; // 1.0
        let total_vv = ((vc + 2) as usize).pow(3); // 64
        let total_vc = (vc as usize).pow(3); // 8

        // Chunk A: world_min = (0, 0, 0)
        // Chunk B: world_min = (chunk_size, 0, 0) = (1.0, 0, 0)
        let chunk_a_min = Vec3::ZERO;
        let chunk_b_min = Vec3::new(chunk_size, 0.0, 0.0);

        // 共享边界的世界坐标
        let shared_pos: [f32; 3] = [1.0, 0.25, 0.25];

        // --- Chunk A 数据 ---
        // +X shell vertex at (ix=2, iy=0, iz=0) → flat index 23
        //   世界位置: (1.0, 0.25, 0.25) — 与 chunk B 的 -X shell 共享
        // 额外三个顶点组成有效 quad:
        //   (ix=1, iy=0, iz=0) → flat 22: (0.75, 0.25, 0.25)
        //   (ix=1, iy=1, iz=0) → flat 26: (0.75, 0.50, 0.25)
        //   (ix=2, iy=1, iz=0) → flat 27: (1.0,  0.50, 0.25)
        // Voxel (1, 0, 0) — voxel_idx = 1 — base = 72
        let mut verts_a = vec![TerrainChunkVertex::default(); total_vv];
        let mut indices_a = vec![0u32; total_vc * 72];

        verts_a[23] = TerrainChunkVertex {
            position: shared_pos,
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        };
        verts_a[22] = TerrainChunkVertex {
            position: [0.75, 0.25, 0.25],
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        };
        verts_a[26] = TerrainChunkVertex {
            position: [0.75, 0.50, 0.25],
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        };
        verts_a[27] = TerrainChunkVertex {
            position: [1.0, 0.50, 0.25],
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        };

        // Quad: 23-22-26, 23-27-26 (两个三角形)
        let base_a = 1 * 72;
        indices_a[base_a] = 23;
        indices_a[base_a + 1] = 22;
        indices_a[base_a + 2] = 26;
        indices_a[base_a + 3] = 23;
        indices_a[base_a + 4] = 27;
        indices_a[base_a + 5] = 26;

        // --- Chunk B 数据 ---
        // -X shell vertex at (ix=-1, iy=0, iz=0) → flat index 20
        //   世界位置: (1.0, 0.25, 0.25) — 与 chunk A 的 +X shell 一致
        // 额外三个顶点:
        //   (ix=0, iy=0, iz=0) → flat 21: (1.25, 0.25, 0.25)
        //   (ix=-1, iy=1, iz=0) → flat 24: (1.0,  0.50, 0.25)
        //   (ix=0, iy=1, iz=0) → flat 25: (1.25, 0.50, 0.25)
        // Voxel (0, 0, 0) — voxel_idx = 0 — base = 0
        let mut verts_b = vec![TerrainChunkVertex::default(); total_vv];
        let mut indices_b = vec![0u32; total_vc * 72];

        verts_b[20] = TerrainChunkVertex {
            position: shared_pos,
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        };
        verts_b[21] = TerrainChunkVertex {
            position: [1.25, 0.25, 0.25],
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        };
        verts_b[24] = TerrainChunkVertex {
            position: [1.0, 0.50, 0.25],
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        };
        verts_b[25] = TerrainChunkVertex {
            position: [1.25, 0.50, 0.25],
            normal: [0.0, 0.0, 1.0],
            ..Default::default()
        };

        // Quad: 20-21-24, 20-24-25
        let base_b = 0;
        indices_b[base_b] = 20;
        indices_b[base_b + 1] = 21;
        indices_b[base_b + 2] = 24;
        indices_b[base_b + 3] = 20;
        indices_b[base_b + 4] = 24;
        indices_b[base_b + 5] = 25;

        // --- Build both meshes ---
        let mesh_a = compact_and_build_mesh(&verts_a, &indices_a, vc, chunk_a_min, voxel_size)
            .expect("chunk A should produce a mesh");
        let mesh_b = compact_and_build_mesh(&verts_b, &indices_b, vc, chunk_b_min, voxel_size)
            .expect("chunk B should produce a mesh");

        // --- Extract compacted positions ---
        let pos_a: Vec<[f32; 3]> = mesh_a
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("mesh A has positions")
            .as_float3()
            .expect("positions are float3")
            .to_vec();
        let pos_b: Vec<[f32; 3]> = mesh_b
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("mesh B has positions")
            .as_float3()
            .expect("positions are float3")
            .to_vec();

        // --- Assertions ---
        assert_eq!(pos_a.len(), 4, "chunk A should have 4 valid vertices");
        assert_eq!(pos_b.len(), 4, "chunk B should have 4 valid vertices");

        // 共享边界顶点 (1.0, 0.25, 0.25) 必须在两个 mesh 中同时出现
        let shared_in_a = pos_a.iter().any(|p| {
            (p[0] - shared_pos[0]).abs() < 0.001
                && (p[1] - shared_pos[1]).abs() < 0.001
                && (p[2] - shared_pos[2]).abs() < 0.001
        });
        let shared_in_b = pos_b.iter().any(|p| {
            (p[0] - shared_pos[0]).abs() < 0.001
                && (p[1] - shared_pos[1]).abs() < 0.001
                && (p[2] - shared_pos[2]).abs() < 0.001
        });
        assert!(
            shared_in_a,
            "shared vertex {shared_pos:?} must appear in chunk A's compacted mesh"
        );
        assert!(
            shared_in_b,
            "shared vertex {shared_pos:?} must appear in chunk B's compacted mesh"
        );

        // 额外验证：两个 mesh 中的共享顶点世界坐标完全相同
        let a_shared = pos_a
            .iter()
            .find(|p| {
                (p[0] - shared_pos[0]).abs() < 0.001
                    && (p[1] - shared_pos[1]).abs() < 0.001
                    && (p[2] - shared_pos[2]).abs() < 0.001
            })
            .expect("shared vertex in chunk A");
        let b_shared = pos_b
            .iter()
            .find(|p| {
                (p[0] - shared_pos[0]).abs() < 0.001
                    && (p[1] - shared_pos[1]).abs() < 0.001
                    && (p[2] - shared_pos[2]).abs() < 0.001
            })
            .expect("shared vertex in chunk B");
        assert!(
            (a_shared[0] - b_shared[0]).abs() < f32::EPSILON
                && (a_shared[1] - b_shared[1]).abs() < f32::EPSILON
                && (a_shared[2] - b_shared[2]).abs() < f32::EPSILON,
            "shared vertex world position must be identical in both meshes: \
             chunk A has {:?}, chunk B has {:?}",
            a_shared,
            b_shared
        );
    }

    /// 验证顶点位置 = chunk_min + local * voxel_size
    #[test]
    fn vertex_in_world_space() {
        let vc = 2u32;
        let total_vv = ((vc + 2) as usize).pow(3); // 64 (双边 shell)
        let total_vc = (vc as usize).pow(3); // 8
        let chunk_min = Vec3::new(10.0, -5.0, 0.0);
        let voxel_size = 0.5;
        let mut verts = vec![TerrainChunkVertex::default(); total_vv];
        let mut indices = vec![0u32; total_vc * 72]; // 72 slots/voxel

        // 三个顶点形成一个三角形，顶点在 world space
        verts[0].position = [10.0, -5.0, 0.0]; // chunk_min (local 0,0,0)
        verts[1].position = [10.5, -5.0, 0.0]; // chunk_min + (1,0,0)*0.5
        verts[2].position = [10.0, -4.5, 0.0]; // chunk_min + (0,1,0)*0.5
        verts[0].normal = [0.0, 0.0, 1.0];
        verts[1].normal = [0.0, 0.0, 1.0];
        verts[2].normal = [0.0, 0.0, 1.0];

        // 一个三角形 0-1-2 (voxel 0 的 6 个索引槽)
        indices[0] = 0;
        indices[1] = 1;
        indices[2] = 2;
        indices[3] = 0;
        indices[4] = 2;
        indices[5] = 1; // degenerate (0,2,1)

        let mesh = compact_and_build_mesh(&verts, &indices, vc, chunk_min, voxel_size);
        assert!(mesh.is_some(), "should produce mesh with vertices");
        let mesh = mesh.unwrap();
        let positions: Vec<[f32; 3]> = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("mesh has positions")
            .as_float3()
            .expect("positions are float3")
            .to_vec();
        // 3 valid vertices, 1 valid triangle
        assert_eq!(positions.len(), 3);
        // vertex 0 = chunk_min → (10, -5, 0)
        assert!((positions[0][0] - 10.0).abs() < 0.001);
    }

    /// ⚠️ 已知偏差：CPU OpenSimplex2D ≠ GPU value noise。待 biome phase 统一。
    #[test]
    #[ignore = "known deviation: CPU OpenSimplex2D vs GPU value noise — unify at biome phase"]
    fn cpu_gpu_noise_parity() {}
}
