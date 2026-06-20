//! GPU compute 管线使用的数据类型定义。
//! 包括 uniform 输入（TerrainChunkInfo）、顶点输出（TerrainChunkVertex）和计数器读回（TerrainChunkCounters）。
//! 所有类型均实现 ShaderType / Pod / Zeroable，可直接映射到 wgpu buffer。

use bevy::render::render_resource::ShaderType;
use bytemuck::{Pod, Zeroable};

/// GPU compute 的每个 chunk 输入参数，含 uniform buffer 布局。
#[derive(ShaderType, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct TerrainChunkInfo {
    /// chunk 最小角的世界坐标
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
    /// 顶点位置（物体空间）
    pub position: [f32; 3],
    /// 16 字节对齐填充
    pub _pad0: u32,
    /// 法线
    pub normal: [f32; 3],
    /// 16 字节对齐填充
    pub _pad1: u32,
}

/// GPU 计数器读回值（vertex_count, index_count）
#[derive(ShaderType, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct TerrainChunkCounters {
    /// 实际顶点数量
    pub vertex_count: u32,
    /// 实际索引数量
    pub index_count: u32,
    /// 16 字节对齐填充
    pub _pad: [u32; 2],
}
