use bevy::render::render_resource::ShaderType;
use bytemuck::{Pod, Zeroable};

#[derive(ShaderType, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct TerrainChunkInfo {
    pub chunk_min: [f32; 3],
    /// 单个 voxel 在世界空间的边长
    pub voxel_size: f32,
    /// 每条边的 voxel 数量
    pub voxel_count: u32,
    /// 整个地形水平范围，用于 UV / 裁剪
    pub terrain_size: f32,
    /// 噪声种子
    pub seed: u32,
    /// 显式填充至 16 字节对齐
    pub _pad: [u32; 2],
}

/// GPU compute 输出的单个顶点
#[derive(ShaderType, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct TerrainChunkVertex {
    pub position: [f32; 3],
    pub _pad0: u32,
    pub normal: [f32; 3],
    pub _pad1: u32,
}

/// GPU atomic 计数器读回值
#[derive(ShaderType, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct TerrainChunkCounters {
    pub vertex_count: u32,
    pub index_count: u32,
    pub _pad: [u32; 2],
}
