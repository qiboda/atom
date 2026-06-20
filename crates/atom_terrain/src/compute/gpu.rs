//! GPU compute mesh generation — Bevy 0.19.
//! Four-pass Dual Contouring on GPU with fixed-slot vertices/indices.
//!
//! State machine per chunk:
//!   0(allocate)→1(pass1)→2(pass2)→3(pass3)→4(pass4)→5(staging copy)→6(readback)→7(cleanup)

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
            BindGroupLayoutDescriptor, BindGroupLayoutEntries, Buffer,
            BufferDescriptor, BufferUsages, CachedComputePipelineId,
            ComputePassDescriptor, ComputePipelineDescriptor, PipelineCache,
            ShaderStages,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
    },
};

use super::{
    sync::TerrainChunksToProcess,
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
        pass3: mk("pass3", "shaders/terrain/compute/main_mesh_compute_vertices.wgsl"),
        pass4: mk("pass4", "shaders/terrain/compute/main_mesh_compute_indices.wgsl"),
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
    /// 计数器映射完成标志
    mapped: Arc<AtomicBool>,
    /// chunk 世界坐标偏移
    world_min: Vec3,
}

/// 待 readback 的 staging buffer 集合 (entity → staging)
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
        let g = (vc + 1) as u64 * (vc + 1) as u64 * (vc + 1) as u64;
        let v = vc as u64 * vc as u64 * vc as u64;
        let s = BufferUsages::STORAGE | BufferUsages::COPY_DST;
        let so = BufferUsages::STORAGE | BufferUsages::COPY_SRC;
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
        let db = mk("density", g * 4, s);
        let cb = mk("cross", v * 12 * 32, s);
        let vb = mk("verts", v * size_of::<TerrainChunkVertex>() as u64, so);
        let ib = mk("indices", v * 6 * 4, so);
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
    mut to_process: ResMut<TerrainChunksToProcess>,
    mut buffers: ResMut<TerrainChunkMeshBuffers>,
    mut progress: ResMut<TerrainChunkComputeProgress>,
    mut staging: ResMut<TerrainChunkStagingBuffers>,
    sender: Res<TerrainChunkMeshSender>,
) {
    let vc = setting.voxel_count;

    // 0. allocate new chunks, then clear from pending (world_min stored in ChunkBuffers)
    let mut to_remove: Vec<Entity> = Vec::new();
    for (&entity, world_min) in to_process.pending.iter() {
        if progress.pass.contains_key(&entity) {
            continue;
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
        to_remove.push(entity);
    }
    for entity in to_remove {
        to_process.pending.remove(&entity);
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
                let n = vc + 1;
                (n.div_ceil(8), n.div_ceil(8), n.div_ceil(8))
            }
            _ => {
                let n = vc;
                (n.div_ceil(8), n.div_ceil(8), n.div_ceil(8))
            }
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
    let v = vc as u64 * vc as u64 * vc as u64;
    let vertex_size = v * size_of::<TerrainChunkVertex>() as u64;
    let index_size = v * 6 * 4;

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
        let Some(s) = staging.buffers.get(&entity) else {
            warn!("pass==6 but no staging buffer for {entity:?}");
            continue;
        };

        // 首次进入 pass==6：发起 async 映射
        if !s.mapped.load(Ordering::Acquire) {
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
            let _ = wgpu_device.poll(
                bevy::render::render_resource::PollType::Poll,
            );
        }

        if !s.mapped.load(Ordering::Acquire) {
            continue; // 映射尚未完成，下帧再试
        }

        let counter_view = s.counters.slice(..).get_mapped_range();
        let _counters: &[u32; 4] = bytemuck::from_bytes(&counter_view[..16]);
        drop(counter_view);
        s.counters.unmap();

        // 读顶点
        s.vertices.slice(..).map_async(
            bevy::render::render_resource::MapMode::Read,
            |_| {},
        );
        // 读索引
        s.indices.slice(..).map_async(
            bevy::render::render_resource::MapMode::Read,
            |_| {},
        );
        let _ = wgpu_device.poll(
            bevy::render::render_resource::PollType::Poll,
        );

        let vertex_view = s.vertices.slice(..).get_mapped_range();
        let all_vertices: &[TerrainChunkVertex] =
            bytemuck::cast_slice(&vertex_view[..s.vertex_size as usize]);
        let index_view = s.indices.slice(..).get_mapped_range();
        let all_indices: &[u32] =
            bytemuck::cast_slice(&index_view[..s.index_size as usize]);

        // compact + remap: 过滤零顶点，重映射索引
        let mesh = compact_and_build_mesh(all_vertices, all_indices, vc);
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

/// 将 GPU 读回的稀疏顶点/索引 compact + remap 为 Bevy Mesh。
/// 过滤零顶点（未生成几何的 voxel），重映射索引，构建 TriangleList mesh。
fn compact_and_build_mesh(
    all_vertices: &[TerrainChunkVertex],
    all_indices: &[u32],
    vc: u32,
) -> Option<Mesh> {
    let total = (vc as usize).pow(3);

    // 构建 old→new 映射：标记有 position 的顶点
    let mut remap: Vec<Option<u32>> = vec![None; total];
    let mut compact_verts: Vec<[f32; 3]> = Vec::new();
    let mut compact_norms: Vec<[f32; 3]> = Vec::new();

    for (i, v) in all_vertices.iter().enumerate().take(total) {
        let len = (v.position[0] * v.position[0]
            + v.position[1] * v.position[1]
            + v.position[2] * v.position[2])
            .sqrt();
        if len > 0.0001 {
            remap[i] = Some(compact_verts.len() as u32);
            compact_verts.push(v.position);
            compact_norms.push(v.normal);
        }
    }

    if compact_verts.is_empty() {
        return None;
    }

    // 遍历索引：每个 voxel 最多 6 个索引 → 两个三角形 (0-1-2, 0-2-3)
    let mut tri_indices: Vec<u32> = Vec::new();
    for voxel_idx in 0..total {
        let base = voxel_idx * 6;
        if base + 5 >= all_indices.len() {
            break;
        }
        let i0 = all_indices[base] as usize;
        let i1 = all_indices[base + 1] as usize;
        let i2 = all_indices[base + 2] as usize;
        let i3 = all_indices[base + 3] as usize;
        let i4 = all_indices[base + 4] as usize;
        let i5 = all_indices[base + 5] as usize;

        // 检查两组三角形是否有效（所有 6 个索引都是有效顶点）
        let r0 = remap.get(i0).copied().flatten();
        let r1 = remap.get(i1).copied().flatten();
        let r2 = remap.get(i2).copied().flatten();
        let r3 = remap.get(i3).copied().flatten();
        let r4 = remap.get(i4).copied().flatten();
        let r5 = remap.get(i5).copied().flatten();

        if let (Some(r0), Some(r1), Some(r2), Some(r3), Some(r4), Some(r5)) =
            (r0, r1, r2, r3, r4, r5)
        {
            // 过滤退化的三角形（三个顶点都不同）
            if r0 != r1 && r0 != r2 && r1 != r2 {
                tri_indices.extend_from_slice(&[r0, r1, r2]);
            }
            if r3 != r4 && r3 != r5 && r4 != r5 {
                tri_indices.extend_from_slice(&[r3, r4, r5]);
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
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, compact_verts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, compact_norms);
    mesh.insert_indices(bevy::mesh::Indices::U32(tri_indices));
    Some(mesh)
}
