//! GPU compute 管线使用的数据类型定义。
//! 包括 uniform 输入（TerrainChunkInfo）、顶点输出（TerrainChunkVertex）和计数器读回（TerrainChunkCounters）。
//! 所有类型均实现 ShaderType / Pod / Zeroable，可直接映射到 wgpu buffer。

use bevy::render::render_resource::ShaderType;
use bytemuck::{Pod, Zeroable};

/// GPU compute 的每个 chunk 输入参数，含 uniform buffer 布局。
/// WGSL uniform 地址空间要求: vec3 对齐 16 字节，struct 总大小 16 的倍数。
/// 布局: chunk_min(12)+pad0(4)=16, voxel_size(4)+voxel_count(4)+terrain_size(4)+seed(4)=16, pad1(8)=8, pad2(8)=8 → 48
#[derive(ShaderType, Clone, Copy, Default, Pod, Zeroable)]
#[repr(C)]
pub struct TerrainChunkInfo {
    /// chunk 最小角的世界坐标 (`vec3<f32>` → WGSL uniform 对齐到 16 字节)
    pub chunk_min: [f32; 3],
    /// uniform vec3 对齐填充
    pub pad0: u32,
    /// 单个 voxel 在世界空间的边长
    pub voxel_size: f32,
    /// 每条边的 voxel 数量
    pub voxel_count: u32,
    /// 整个地形水平范围，用于 UV / 裁剪
    pub terrain_size: f32,
    /// 噪声种子
    pub seed: u32,
    /// 填充至 32 字节
    pub pad1: [u32; 2],
    /// 填充至 48 字节（WGSL uniform struct 大小为 16 的倍数）
    pub pad2: [u32; 2],
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

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-5,
            "expected {a} ≈ {b} but diff = {}",
            (a - b).abs()
        );
    }

    // ── TerrainChunkInfo ──

    #[test]
    fn chunk_info_default_all_zero() {
        let info = TerrainChunkInfo::default();
        assert_eq!(info.chunk_min, [0.0; 3]);
        assert_eq!(info.pad0, 0);
        assert_approx(info.voxel_size, 0.0);
        assert_eq!(info.voxel_count, 0);
        assert_approx(info.terrain_size, 0.0);
        assert_eq!(info.seed, 0);
        assert_eq!(info.pad1, [0; 2]);
        assert_eq!(info.pad2, [0; 2]);
    }

    #[test]
    fn chunk_info_fields_roundtrip() {
        let info = TerrainChunkInfo {
            chunk_min: [1.0, 2.0, 3.0],
            voxel_size: 0.5,
            voxel_count: 30,
            terrain_size: 4096.0,
            seed: 42,
            ..Default::default()
        };
        assert_eq!(info.chunk_min, [1.0, 2.0, 3.0]);
        assert_approx(info.voxel_size, 0.5);
        assert_eq!(info.voxel_count, 30);
        assert_approx(info.terrain_size, 4096.0);
        assert_eq!(info.seed, 42);
    }

    #[test]
    fn chunk_info_uniform_layout_48_bytes() {
        assert_eq!(std::mem::size_of::<TerrainChunkInfo>(), 48);
        assert_eq!(TerrainChunkInfo::min_size().get(), 48);
    }

    // ── TerrainChunkVertex ──

    #[test]
    fn vertex_default_all_zero() {
        let v = TerrainChunkVertex::default();
        assert_eq!(v.position, [0.0; 3]);
        assert_eq!(v._pad0, 0);
        assert_eq!(v.normal, [0.0; 3]);
        assert_eq!(v._pad1, 0);
    }

    #[test]
    fn vertex_fields_roundtrip() {
        let v = TerrainChunkVertex {
            position: [1.5, -2.0, 3.25],
            normal: [0.0, 1.0, 0.0],
            ..Default::default()
        };
        assert_eq!(v.position, [1.5, -2.0, 3.25]);
        assert_eq!(v.normal, [0.0, 1.0, 0.0]);
    }

    #[test]
    fn vertex_layout_32_bytes() {
        assert_eq!(std::mem::size_of::<TerrainChunkVertex>(), 32);
        assert_eq!(TerrainChunkVertex::min_size().get(), 32);
    }

    // ── TerrainChunkCounters ──

    #[test]
    fn counters_default_all_zero() {
        let c = TerrainChunkCounters::default();
        assert_eq!(c.vertex_count, 0);
        assert_eq!(c.index_count, 0);
        assert_eq!(c._pad, [0; 2]);
    }

    #[test]
    fn counters_fields_roundtrip() {
        let c = TerrainChunkCounters {
            vertex_count: 1234,
            index_count: 5678,
            _pad: [9, 10],
        };
        assert_eq!(c.vertex_count, 1234);
        assert_eq!(c.index_count, 5678);
        assert_eq!(c._pad, [9, 10]);
    }

    #[test]
    fn counters_layout_16_bytes() {
        assert_eq!(std::mem::size_of::<TerrainChunkCounters>(), 16);
        assert_eq!(TerrainChunkCounters::min_size().get(), 16);
    }
}
