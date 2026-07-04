//! Per-chunk GPU terrain compute pipeline.
//!
//! Per-slot buffers, shared pipeline objects, 6-pass state machine.

use bevy::{
    prelude::*,
    render::{
        render_resource::{
            BindGroup, BindGroupEntry, BindGroupLayoutDescriptor,
            BindGroupLayoutEntries, Buffer, BufferDescriptor, BufferUsages,
            CachedComputePipelineId, ComputePassDescriptor, ComputePipelineDescriptor,
            PipelineCache, ShaderStages, ShaderType, binding_types::*,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
    },
};

use super::chunk::{ChunkId, ChunkLoadRequest, ChunkManager};
use super::types::TerrainChunkVertex;

const GRID_SIZE: u32 = 32;
const VOXEL_SIZE: f32 = 0.5;
const MAX_SLOTS: usize = 512;

fn slot_bytes() -> (u64, u64, u64, u64, u64) {
    let gs = GRID_SIZE as u64;
    let n = gs + 2;
    let vc = gs * gs * gs;
    (n * n * n * 4, vc * 12 * 8 * 4, vc * 4, vc * 32, vc * 12 * 6 * 4)
}

#[repr(C)]
#[derive(Clone, Copy, ShaderType, bytemuck::Pod, bytemuck::Zeroable)]
struct ChunkUniform {
    grid_min: [f32; 3],
    pad0: u32,           // → 16
    voxel_size: f32,
    grid_size: u32,
    pad1: [u32; 2],      // → 32
    neighbor_mask: u32,
    pad3: u32,           // → 40
    pad4: [u32; 2],      // → 48
}

/// Per-chunk GPU compute slot
pub struct GpuSlot {
    /// Compute uniform buffer (grid_min, voxel_size, grid_size)
    pub uniform: Buffer,
    /// Render uniform buffer (view-projection Mat4)
    pub uniform_render: Buffer,
    /// SDF density buffer (34³ f32)
    pub sdf: Buffer,
    /// Cross-point buffer
    pub cross: Buffer,
    /// Voxel allocation buffer (32³ u32)
    pub voxel_alloc: Buffer,
    /// Vertex buffer (output from compute)
    pub vertices: Buffer,
    /// Index buffer (output from compute)
    pub indices: Buffer,
    /// Indirect draw command buffer
    pub indirect: Buffer,
    /// Counter buffer (vertex_count, index_count)
    pub counters: Buffer,
    /// Compute bind group (8 entries)
    pub bg_compute: BindGroup,
    /// Render bind group (1 uniform)
    pub bg_render: BindGroup,
    /// Current chunk ID (None = slot available)
    pub chunk_id: Option<ChunkId>,
    /// Current pass index (0..5 = computing, 6 = ready)
    pub pass: u32,
}
#[derive(Resource)]
/// Per-chunk compute pipeline resource
pub struct PerChunkComputePipeline {
    /// 6 compute pipeline IDs
    pub passes: [CachedComputePipelineId; 6],
    /// All GPU slots
    pub slots: Vec<Option<GpuSlot>>,
    /// Free slot indices (LIFO)
    pub free: Vec<usize>,
}

fn make_buf(device: &RenderDevice, label: &str, size: u64, usage: BufferUsages) -> Buffer {
    device.create_buffer(&BufferDescriptor { label: Some(label), size, usage, mapped_at_creation: false })
}

/// Initialize per-chunk compute pipeline
pub fn init_per_chunk_compute(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
) {
    let entries = BindGroupLayoutEntries::sequential(ShaderStages::COMPUTE, (
        uniform_buffer::<ChunkUniform>(false), storage_buffer::<Vec<f32>>(false),
        storage_buffer::<Vec<u32>>(false), storage_buffer::<Vec<u32>>(false),
        storage_buffer::<Vec<TerrainChunkVertex>>(false), storage_buffer::<Vec<u32>>(false),
        storage_buffer::<Vec<u32>>(false), storage_buffer::<Vec<u32>>(false),
    ));
    let bgl_desc = BindGroupLayoutDescriptor::new("pc_bgl_compute", &entries);
    let bgl = render_device.create_bind_group_layout("pc_bgl_compute", &entries);
    let r_entries = BindGroupLayoutEntries::sequential(ShaderStages::VERTEX, (uniform_buffer::<Mat4>(false),));
    let r_bgl = render_device.create_bind_group_layout("pc_bgl_render", &r_entries);

    let (sdf_sz, cross_sz, va_sz, vert_sz, idx_sz) = slot_bytes();
    let mut slots: Vec<Option<GpuSlot>> = (0..MAX_SLOTS).map(|_| None).collect();
    let free: Vec<usize> = (0..MAX_SLOTS).rev().collect();

    for (i, slot) in slots.iter_mut().enumerate() {
        let tag = format!("pc{i}");
        let uniform = make_buf(&render_device, &format!("{tag}_uniform"), 48, BufferUsages::UNIFORM | BufferUsages::COPY_DST);
        let uniform_render = make_buf(&render_device, &format!("{tag}_urender"), 64, BufferUsages::UNIFORM | BufferUsages::COPY_DST);
        let sdf = make_buf(&render_device, &format!("{tag}_sdf"), sdf_sz, BufferUsages::STORAGE | BufferUsages::COPY_DST);
        let cross = make_buf(&render_device, &format!("{tag}_cross"), cross_sz, BufferUsages::STORAGE | BufferUsages::COPY_DST);
        let voxel_alloc = make_buf(&render_device, &format!("{tag}_va"), va_sz, BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST);
        let vertices = make_buf(&render_device, &format!("{tag}_vert"), vert_sz, BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::VERTEX | BufferUsages::COPY_DST);
        let indices = make_buf(&render_device, &format!("{tag}_idx"), idx_sz, BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::INDEX | BufferUsages::COPY_DST);
        let counters = make_buf(&render_device, &format!("{tag}_ctr"), 16, BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST);
        let indirect = make_buf(&render_device, &format!("{tag}_ind"), 24, BufferUsages::STORAGE | BufferUsages::INDIRECT);

        let lbl = format!("{tag}_bgc");
        let bg_compute = render_device.create_bind_group(lbl.as_str(), &bgl, &[
            BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() },
            BindGroupEntry { binding: 1, resource: sdf.as_entire_binding() },
            BindGroupEntry { binding: 2, resource: cross.as_entire_binding() },
            BindGroupEntry { binding: 3, resource: voxel_alloc.as_entire_binding() },
            BindGroupEntry { binding: 4, resource: vertices.as_entire_binding() },
            BindGroupEntry { binding: 5, resource: counters.as_entire_binding() },
            BindGroupEntry { binding: 6, resource: indices.as_entire_binding() },
            BindGroupEntry { binding: 7, resource: indirect.as_entire_binding() },
        ]);
        let lbl = format!("{tag}_bgr");
        let bg_render = render_device.create_bind_group(lbl.as_str(), &r_bgl, &[
            BindGroupEntry { binding: 0, resource: uniform_render.as_entire_binding() },
        ]);
        *slot = Some(GpuSlot {
            uniform, uniform_render, sdf, cross, voxel_alloc, vertices, indices, indirect, counters,
            bg_compute, bg_render, chunk_id: None, pass: 0,
        });
    }

    let pass_data = [
        ("pc_sdf", "shaders/terrain/compute/sdf_fill.wgsl"),
        ("pc_edge", "shaders/terrain/compute/edge_detect.wgsl"),
        ("pc_alloc", "shaders/terrain/compute/vertex_alloc.wgsl"),
        ("pc_qef", "shaders/terrain/compute/qef_solve.wgsl"),
        ("pc_idx", "shaders/terrain/compute/index_build.wgsl"),
        ("pc_indirect", "shaders/terrain/compute/fill_indirect.wgsl"),
    ];
    let passes = pass_data.map(|(label, path)| {
        pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(label.into()), layout: vec![bgl_desc.clone()], shader: asset_server.load(path), ..default()
        })
    });
    commands.insert_resource(PerChunkComputePipeline { passes, slots, free });
}

/// Dispatch compute passes for all active chunks
pub fn per_chunk_compute_system(
    pipeline: Res<PerChunkComputePipeline>, manager: Res<ChunkManager>,
    cache: Res<PipelineCache>, queue: Res<RenderQueue>, mut ctx: RenderContext,
) {
    let encoder = ctx.command_encoder();
    for (&_cid, &slot_idx) in &manager.active {
        let Some(slot) = &pipeline.slots[slot_idx] else { continue };
        if slot.pass != 0 { continue; }
        encoder.clear_buffer(&slot.sdf, 0, None);
        encoder.clear_buffer(&slot.cross, 0, None);
        encoder.clear_buffer(&slot.voxel_alloc, 0, None);
        encoder.clear_buffer(&slot.vertices, 0, None);
        encoder.clear_buffer(&slot.indices, 0, None);
    }
    let mut pass = encoder.begin_compute_pass(&ComputePassDescriptor { label: Some("pc_compute"), ..default() });
    for (&cid, &slot_idx) in &manager.active {
        let Some(slot) = &pipeline.slots[slot_idx] else { continue };
        if slot.pass >= 6 { continue; }
        let min = cid.world_min();
        let mask = manager.neighbor_mask(&cid);
        queue.write_buffer(&slot.uniform, 0, bytemuck::bytes_of(&ChunkUniform {
            grid_min: min.to_array(), pad0: 0, voxel_size: VOXEL_SIZE, grid_size: GRID_SIZE,
            pad1: [0; 2], neighbor_mask: mask, pad3: 0, pad4: [0; 2],
        }));
        if slot.pass == 0 {
            let zero: [u32; 4] = [0, 0, 0, 0];
            queue.write_buffer(&slot.counters, 0, bytemuck::bytes_of(&zero));
        }
        let pid = pipeline.passes[slot.pass as usize];
        let Some(pso) = cache.get_compute_pipeline(pid) else { continue };
        pass.set_pipeline(pso);
        pass.set_bind_group(0, &slot.bg_compute, &[]);
        if slot.pass == 0 {
            pass.dispatch_workgroups(GRID_SIZE + 2, GRID_SIZE + 2, GRID_SIZE + 2);
        } else if slot.pass == 5 {
            pass.dispatch_workgroups(1, 1, 1);
        } else {
            pass.dispatch_workgroups(GRID_SIZE, GRID_SIZE, GRID_SIZE);
        }
    }
}

/// Advance per-chunk compute states
pub fn advance_chunk_states(
    mut pipeline: ResMut<PerChunkComputePipeline>, manager: Res<ChunkManager>, cache: Res<PipelineCache>,
) {
    for (&cid, &slot_idx) in &manager.active {
        let current_pass = pipeline.slots[slot_idx].as_ref().map(|s| s.pass).unwrap_or(6);
        if current_pass < 6 {
            let pid = pipeline.passes[current_pass as usize];
            if cache.get_compute_pipeline(pid).is_some() {
                if let Some(slot) = &mut pipeline.slots[slot_idx] {
                    slot.chunk_id.get_or_insert(cid);
                    slot.pass += 1;
                }
            }
        }
    }
}

/// Main world: update ChunkManager based on observer
pub fn chunk_management_system(
    observer: Res<super::global_compute::TerrainObserver>,
    mut manager: ResMut<ChunkManager>, mut req: ResMut<ChunkLoadRequest>,
) { manager.update_for_observer(observer.position, &mut req); }

/// Render world: process load/unload requests
pub fn slot_sync_system(
    mut pipeline: ResMut<PerChunkComputePipeline>, mut manager: ResMut<ChunkManager>, req: Res<ChunkLoadRequest>,
) {
    let to_unload: Vec<ChunkId> = manager.active.keys().filter(|cid| !req.wanted.contains(cid)).copied().collect();
    for cid in to_unload {
        if let Some(slot_idx) = manager.active.remove(&cid) {
            if let Some(slot) = &mut pipeline.slots[slot_idx] { slot.chunk_id = None; slot.pass = 0; }
            pipeline.free.push(slot_idx);
        }
    }
    for cid in req.wanted.iter() {
        if manager.active.contains_key(cid) { continue; }
        if let Some(slot_idx) = pipeline.free.pop() {
            if let Some(slot) = &mut pipeline.slots[slot_idx] { slot.chunk_id = Some(*cid); slot.pass = 0; }
            manager.active.insert(*cid, slot_idx);
        } else {
            bevy::log::warn!("No free GPU slot for chunk {:?} (MAX_SLOTS={})", cid, MAX_SLOTS);
        }
    }
}
