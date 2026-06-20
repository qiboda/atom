//! GPU compute mesh generation — Bevy 0.19.
//! Four-pass Dual Contouring on GPU with fixed-slot vertices/indices.
//!
//! State machine per chunk: 0→allocate→1→2→3→4→5(readback)→done

use std::collections::HashMap;

use bevy::{
    prelude::*,
    render::{
        render_resource::{binding_types::{storage_buffer, uniform_buffer}, *},
        renderer::{RenderContext, RenderDevice, RenderQueue},
    },
};

use crate::{mesh::{TerrainChunkMeshData, TerrainChunkMeshSender}, setting::TerrainSetting};
use super::{sync::TerrainChunksToProcess, types::{TerrainChunkInfo, TerrainChunkVertex}};

/// 地形 compute 管线资源，包含 bind group layout 和四个 pass 的 compute pipeline id。
#[derive(Resource)]
pub struct TerrainComputePipeline {
    /// bind group layout（所有 pass 共用）
    pub bind_group_layout: BindGroupLayout,
    /// pass 1: 密度场计算 (voxel_vertices.wgsl)
    pub pass1: CachedComputePipelineId,
    /// pass 2: 边交叉点二分查找 (voxel_cross_points.wgsl)
    pub pass2: CachedComputePipelineId,
    /// pass 3: QEF 顶点计算 (main_mesh_compute_vertices.wgsl)
    pub pass3: CachedComputePipelineId,
    /// pass 4: 三角形索引生成 (main_mesh_compute_indices.wgsl)
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
    /// 密度场值 (f32 per grid point)
    pub density: Buffer,
    /// 边交叉点 (32 bytes per edge)
    pub cross_points: Buffer,
    /// 顶点 buffer (TerrainChunkVertex per voxel)
    pub vertices: Buffer,
    /// 索引 buffer (u32 per index slot)
    pub indices: Buffer,
    /// 统计计数器 (4×u32)
    pub counters: Buffer,
    /// bind group（绑定上述所有 buffer + chunk_info uniform）
    pub bind_group: BindGroup,
}

/// 所有 chunk 的 GPU buffer 映射表（Entity → ChunkBuffers）。
/// 作为资源注入，由 terrain_compute_system 管理生命周期。
#[derive(Resource, Default)]
pub struct TerrainChunkMeshBuffers { buffers: HashMap<Entity, ChunkBuffers> }

/// 每个 chunk 的 compute pass 进度计数器。
/// 0=未开始，1-4=pass 序号，5=等待读回。
/// 由 terrain_compute_system 每帧推进。
#[derive(Resource, Default)]
pub struct TerrainChunkComputeProgress {
    /// entity → 当前 pass
    pub pass: HashMap<Entity, u32>,
}

impl TerrainChunkMeshBuffers {
    fn allocate(&mut self, entity: Entity, info: &TerrainChunkInfo, vc: u32,
                device: &RenderDevice, queue: &RenderQueue, bgl: &BindGroupLayout) {
        let g = (vc + 1) as u64 * (vc + 1) as u64 * (vc + 1) as u64;
        let v = vc as u64 * vc as u64 * vc as u64;
        let s = BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC;
        let u = BufferUsages::UNIFORM | BufferUsages::COPY_DST;

        let mk = |label: &str, size: u64, usage: BufferUsages| {
            device.create_buffer(&BufferDescriptor { label: Some(label), size, usage, mapped_at_creation: false })
        };

        let db = mk("density", g * 4, s);
        let cb = mk("cross", v * 12 * 32, s);
        let vb = mk("verts", v * size_of::<TerrainChunkVertex>() as u64, s);
        let ib = mk("indices", v * 6 * 4, s);
        let ct = mk("counters", 16, s | BufferUsages::COPY_DST);
        let ub = mk("chunk_info", size_of::<TerrainChunkInfo>() as u64, u);

        queue.write_buffer(&ub, 0, bytemuck::bytes_of(info));
        let zero: [u32; 4] = [0; 4];
        queue.write_buffer(&ct, 0, bytemuck::bytes_of(&zero));

        let bg = device.create_bind_group("chunk_bg", bgl, &[
            BindGroupEntry { binding: 0, resource: ub.as_entire_binding() },
            BindGroupEntry { binding: 1, resource: db.as_entire_binding() },
            BindGroupEntry { binding: 2, resource: cb.as_entire_binding() },
            BindGroupEntry { binding: 3, resource: vb.as_entire_binding() },
            BindGroupEntry { binding: 4, resource: ib.as_entire_binding() },
            BindGroupEntry { binding: 5, resource: ct.as_entire_binding() },
        ]);

        self.buffers.insert(entity, ChunkBuffers { density: db, cross_points: cb, vertices: vb, indices: ib, counters: ct, bind_group: bg });
    }
    fn get(&self, entity: Entity) -> Option<&ChunkBuffers> { self.buffers.get(&entity) }
    fn remove(&mut self, entity: Entity) -> Option<ChunkBuffers> { self.buffers.remove(&entity) }
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
    to_process: Res<TerrainChunksToProcess>,
    mut buffers: ResMut<TerrainChunkMeshBuffers>,
    mut progress: ResMut<TerrainChunkComputeProgress>,
    sender: Res<TerrainChunkMeshSender>,
) {
    let vc = setting.voxel_count;

    // 0. allocate new chunks
    for (&entity, world_min) in to_process.pending.iter() {
        if progress.pass.contains_key(&entity) { continue; }
        let info = TerrainChunkInfo {
            chunk_min: world_min.to_array(), voxel_size: setting.voxel_size,
            voxel_count: vc, terrain_size: setting.terrain_size, seed: setting.seed,
            _pad: [0; 2],
        };
        buffers.allocate(entity, &info, vc, &device, &queue, &pipeline.bind_group_layout);
        progress.pass.insert(entity, 0);
    }

    // 1. dispatch
    let encoder = render_context.command_encoder();
    for (&entity, pass) in progress.pass.iter() {
        let Some(cb) = buffers.get(entity) else { continue };
        let pid = match *pass { 1 => pipeline.pass1, 2 => pipeline.pass2, 3 => pipeline.pass3, 4 => pipeline.pass4, _ => continue };
        let Some(cp) = pipeline_cache.get_compute_pipeline(pid) else { continue };
        let wg = match *pass {
            1 => { let n = vc + 1; (n.div_ceil(8), n.div_ceil(8), n.div_ceil(8)) }
            _ => { let n = vc; (n.div_ceil(8), n.div_ceil(8), n.div_ceil(8)) }
        };
        let mut cpass = encoder.begin_compute_pass(&ComputePassDescriptor::default());
        cpass.set_pipeline(cp);
        cpass.set_bind_group(0, &cb.bind_group, &[]);
        cpass.dispatch_workgroups(wg.0, wg.1, wg.2);
    }

    // 2. advance
    for pass in progress.pass.values_mut() { if *pass < 4 { *pass += 1; } }

    // 3. readback pass==5
    let done: Vec<Entity> = progress.pass.iter().filter(|(_, p)| **p == 5).map(|(e, _)| *e).collect();
    for entity in done {
        let Some(cb) = buffers.remove(entity) else { progress.pass.remove(&entity); continue; };

        let tv = (vc * vc * vc) as usize;
        let all_verts: Vec<TerrainChunkVertex> = read_buffer_vec(&device, &cb.vertices, (tv * size_of::<TerrainChunkVertex>()) as u64);

        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut remap: Vec<u32> = vec![0u32; tv];
        for (vi, v) in all_verts.iter().enumerate() {
            let l2 = v.position[0]*v.position[0] + v.position[1]*v.position[1] + v.position[2]*v.position[2];
            if l2 > 0.000001 { remap[vi] = positions.len() as u32; positions.push(v.position); normals.push(v.normal); }
        }

        let raw: Vec<u32> = read_buffer_vec(&device, &cb.indices, (tv * 6 * 4) as u64);
        let mut mapped = Vec::new();
        for chunk in raw.chunks(6) {
            if chunk.len() < 6 || (chunk[0] == 0 && chunk[1] == 0 && chunk[2] == 0) { continue; }
            for &vi in &chunk[..6] { if (vi as usize) < remap.len() { mapped.push(remap[vi as usize]); } }
        }

        if positions.is_empty() || mapped.is_empty() { progress.pass.remove(&entity); continue; }

        let mut mesh = Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_indices(bevy::mesh::Indices::U32(mapped));
        let _ = sender.send(TerrainChunkMeshData { mesh, chunk_entity: entity });
        progress.pass.remove(&entity);
    }

    // 4. mark ready
    for pass in progress.pass.values_mut() { if *pass == 4 { *pass = 5; } }
}

// ── buffer read helpers ──

fn poll_and_read(device: &RenderDevice, buffer: &Buffer, size: u64) -> Option<Box<[u8]>> {
    let slice = buffer.slice(..size);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(MapMode::Read, move |r| { let _ = tx.send(r); });
    let _ = device.wgpu_device().poll(PollType::Wait { submission_index: None, timeout: None });
    match rx.recv() { Ok(Ok(())) => { let view = slice.get_mapped_range(); let data = view.to_vec().into_boxed_slice(); drop(view); buffer.unmap(); Some(data) } _ => None }
}

#[allow(dead_code)]
fn read_buffer<T: bytemuck::Pod>(device: &RenderDevice, buffer: &Buffer, size: u64) -> Option<T> {
    poll_and_read(device, buffer, size).and_then(|d| (d.len() >= size as usize).then(|| *bytemuck::from_bytes(&d[..size as usize])))
}

fn read_buffer_vec<T: bytemuck::Pod>(device: &RenderDevice, buffer: &Buffer, size: u64) -> Vec<T> {
    let Some(data) = poll_and_read(device, buffer, size) else { return Vec::new() };
    let n = (data.len() / size_of::<T>()).min(size as usize / size_of::<T>());
    bytemuck::cast_slice::<u8, T>(&data[..n * size_of::<T>()]).to_vec()
}
