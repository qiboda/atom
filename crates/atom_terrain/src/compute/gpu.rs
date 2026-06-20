//! GPU compute mesh generation — Bevy 0.19.
//! Four-pass Dual Contouring on GPU with fixed-slot vertices/indices.

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

#[derive(Resource)]
pub struct TerrainComputePipeline {
    pub bind_group_layout: BindGroupLayout,
    pub pass1: CachedComputePipelineId,
    pub pass2: CachedComputePipelineId,
    pub pass3: CachedComputePipelineId,
    pub pass4: CachedComputePipelineId,
}

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

pub struct ChunkBuffers {
    pub density: Buffer,
    pub cross_points: Buffer,
    pub vertices: Buffer,
    pub indices: Buffer,
    pub counters: Buffer,
    pub bind_group: BindGroup,
}

#[derive(Resource, Default)]
pub struct TerrainChunkMeshBuffers { buffers: HashMap<Entity, ChunkBuffers> }

#[derive(Resource, Default)]
pub struct TerrainChunkComputeProgress { pub pass: HashMap<Entity, u32> }

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
