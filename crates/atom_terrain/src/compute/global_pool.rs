//! 全局 Mesh Pool — 观察者驱动，持久 vertex/index buffer，无 chunk 边界。
//!
//! ## 设计
//!
//! - SDF grid: 以观察者为中心的世界坐标立方体 (grid_size³ 采样点)
//! - Vertex pool: 固定容量 ring buffer，free list 管理 slot 分配/回收
//! - Index pool: 同上
//! - 所有 buffer 持久化（非 per-frame 分配），观察者移动时增量更新

use bevy::{
    prelude::*,
    render::{
        render_resource::{Buffer, BufferDescriptor, BufferUsages},
        renderer::RenderDevice,
    },
};

/// 全局 Mesh 资源池
#[derive(Resource)]
pub struct GlobalMeshPool {
    /// SDF 密度 grid: (grid_size+1)³ 个 f32
    pub sdf: Buffer,
    /// cross points 中间数据: grid_size³ * 12 * 32 bytes
    pub cross: Buffer,
    /// per-voxel vertex index 分配表: grid_size³ 个 u32 (~0u = 无顶点)
    pub voxel_alloc: Buffer,
    /// 顶点 buffer: capacity 个 TerrainChunkVertex (32B)
    pub vertices: Buffer,
    /// 索引 buffer: capacity 个 u32
    pub indices: Buffer,
    /// 计数器 buffer: [vertex_count, index_count, pad, pad] — atomic 可写
    pub counters: Buffer,
    /// Indirect draw command buffer: [index_count, instance_count, first_index,
    /// vertex_offset, first_instance, pad] (24 bytes, DrawIndexedIndirect)
    pub indirect: Buffer,
    /// grid 每轴的采样点数 (含端点)
    pub grid_size: u32,
    /// 顶点容量
    pub vertex_capacity: u32,
    /// 索引容量
    pub index_capacity: u32,
}

impl GlobalMeshPool {
    /// 以观察者为中心的 grid 半边长（世界单位，legacy，不再用于 grid 对齐）
    pub const VIEW_RADIUS: f32 = 16.0;

    /// voxel 边长
    pub fn voxel_size(&self) -> f32 {
        0.5
    }

    /// grid 世界坐标原点（对齐到 grid_world_size 边界，与 observer 无关）。
    pub fn grid_min(&self, observer: Vec3) -> Vec3 {
        let grid_world_size = self.grid_size as f32 * self.voxel_size();
        (observer / grid_world_size).floor() * grid_world_size
    }

    /// 世界坐标 → grid 索引 (clamped)
    pub fn world_to_grid(&self, world: Vec3, observer: Vec3) -> (u32, u32, u32) {
        let min = self.grid_min(observer);
        let rel = (world - min) / self.voxel_size();
        let clamp = |v: f32| v.round().clamp(0.0, self.grid_size as f32) as u32;
        (clamp(rel.x), clamp(rel.y), clamp(rel.z))
    }

    /// grid 索引 → buffer offset
    pub fn grid_offset(&self, gx: u32, gy: u32, gz: u32) -> u32 {
        let n = self.grid_size + 1;
        gx + gy * n + gz * n * n
    }

    /// 创建全局 buffer pool。
    /// `grid_size` 是每条轴的 voxel 数量（不含端点）。
    pub fn new(device: &RenderDevice, grid_size: u32) -> Self {
        let n = grid_size + 1u32;
        let vc = grid_size; // voxel count per axis

        let dg = n as u64 * n as u64 * n as u64; // density grid points
        let cn = vc as u64 * vc as u64 * vc as u64; // cross point slots
        let vertex_cap = cn as u32; // fixed slot: one vertex per voxel
        let index_cap = vertex_cap * 72; // Phase 2: 12 edges × 6 indices per voxel

        let mk = |label: &str, size: u64, usage: BufferUsages| {
            device.create_buffer(&BufferDescriptor {
                label: Some(label),
                size,
                usage,
                mapped_at_creation: false,
            })
        };

        let s = BufferUsages::STORAGE | BufferUsages::COPY_DST;
        let so = BufferUsages::STORAGE | BufferUsages::COPY_DST | BufferUsages::COPY_SRC;
        let sv = so | BufferUsages::VERTEX;
        let si = so | BufferUsages::INDEX;
        let sid = so | BufferUsages::INDIRECT;

        Self {
            sdf: mk("global_sdf", dg * 4, s),
            cross: mk("global_cross", cn * 12 * 32, s),
            voxel_alloc: mk("global_voxel_alloc", cn * 4, so),
            vertices: mk("global_verts", vertex_cap as u64 * 32, sv),
            indices: mk("global_indices", index_cap as u64 * 4, si),
            counters: mk("global_counters", 16, so),
            indirect: mk("global_indirect", 24, sid),
            grid_size,
            vertex_capacity: vertex_cap,
            index_capacity: index_cap,
        }
    }
}
