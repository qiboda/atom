/// 这些类型需要和shader中的struct保持一致
use bevy::{
    math::{UVec2, UVec4, Vec3, Vec4},
    render::render_resource::ShaderType,
};
use bytemuck::{Pod, Zeroable};

use crate::isosurface::IsosurfaceSide;
use bevy::prelude::*;

/**
 * 单个体素的边的相交点信息
 */
#[derive(ShaderType, Default, Clone, Copy, Debug)]
pub struct VoxelEdgeCrossPoint {
    // xyz是相交位置, w是是否存在
    pub cross_pos: Vec4,
    // xyz是法线, w是材质索引
    pub normal_material_index: Vec4,
}

/**
 * 存储了chunk的基本信息，用于compute shader中
 * TODO: 可以考虑把可变的和不可变的拆分，减少更新数据量
 */
#[derive(ShaderType, Default)]
pub struct TerrainChunkInfo {
    // xyz: chunk的最小位置
    // w: terrain总大小（用于 biome UV 计算）
    pub chunk_min_location_size: Vec4,
    // unit: meter
    pub voxel_size: f32,
    // unit: meter
    pub voxel_num: u32,
    // qef_threshold < 0 => 不使用qef
    pub qef_threshold: f32,
    pub qef_stddev: f32,
}

#[repr(C)]
#[derive(ShaderType, Default, Clone, PartialEq, Copy, Debug, Pod, Zeroable)]
pub struct TerrainChunkVertexInfo {
    pub vertex_location: Vec4,
    pub vertex_normal: Vec4,
    pub vertex_local_coord: UVec4,
    pub voxel_biome: UVec2,
    pub voxel_side: UVec2,
}

impl TerrainChunkVertexInfo {
    pub fn is_on_border(&self, voxel_num: u32) -> bool {
        self.vertex_local_coord.x == 0
            || self.vertex_local_coord.y == 0
            || self.vertex_local_coord.z == 0
            || self.vertex_local_coord.x == voxel_num - 1
            || self.vertex_local_coord.y == voxel_num - 1
            || self.vertex_local_coord.z == voxel_num - 1
    }

    pub fn unpack_u32(value: u32) -> [u32; 4] {
        [
            value & 0x000000FF,
            (value & 0x0000FF00) >> 8,
            (value & 0x00FF0000) >> 16,
            (value & 0xFF000000) >> 24,
        ]
    }

    pub fn get_voxel_side(&self) -> [IsosurfaceSide; 8] {
        let x = TerrainChunkVertexInfo::unpack_u32(self.voxel_side.x)
            .map(|x| x > 0)
            .map(IsosurfaceSide::from);
        let y = TerrainChunkVertexInfo::unpack_u32(self.voxel_side.y)
            .map(|x| x > 0)
            .map(IsosurfaceSide::from);
        [x[0], x[1], x[2], x[3], y[0], y[1], y[2], y[3]]
    }

    // pub fn get_vertex_biome(&self) -> GpuTerrainType {
    //     let biomes = self.get_voxel_biome();
    //     select_voxel_biome(biomes)
    // }

    // pub fn get_voxel_biome(&self) -> [GpuTerrainType; 8] {
    //     let x = TerrainChunkVertexInfo::unpack_u32(self.voxel_biome.x)
    //         .map(|x| GpuTerrainType::from_repr(x as usize).expect("Invalid GpuTerrainType repr"));
    //     let y = TerrainChunkVertexInfo::unpack_u32(self.voxel_biome.y)
    //         .map(|x| GpuTerrainType::from_repr(x as usize).expect("Invalid GpuTerrainType repr"));
    //     [x[0], x[1], x[2], x[3], y[0], y[1], y[2], y[3]]
    // }
}

#[repr(C)]
#[derive(ShaderType, Default, Clone, Copy, Debug, Pod, Zeroable)]
pub struct TerrainChunkVerticesIndicesCount {
    pub vertices_count: u32,
    pub indices_count: u32,
}

#[repr(C)]
#[derive(ShaderType, Default, Clone, Copy, Debug, Pod, Zeroable)]
pub struct TerrainChunkCSGOperation {
    pub location: Vec3,
    pub primitive_type: u32,
    pub shape: Vec3,
    pub operation_type: u32,
}

pub const INVALID_TERRAIN_CHUNK_CSG_OPERATION: TerrainChunkCSGOperation =
    TerrainChunkCSGOperation {
        location: Vec3::ZERO,
        primitive_type: 10000,
        shape: Vec3::ZERO,
        operation_type: 10000,
    };

/**
 * 存储了compute shader计算得出的所在顶点的 isosurface 值。
 */
#[derive(ShaderType)]
pub struct VoxelVertexValueVec {
    #[shader(size(runtime))]
    pub values: Vec<f32>,
}

/**
 * 存储了compute shader计算得出的体素的所有边的相交点信息。
 */
#[derive(ShaderType)]
pub struct VoxelEdgeCrossPointVec {
    #[shader(size(runtime))]
    pub cross_points: Vec<VoxelEdgeCrossPoint>,
}

/**
 * 存储了compute shader计算得出的地形块的顶点信息
 */
#[derive(ShaderType)]
pub struct TerrainChunkMeshVertexInfoVec {
    #[shader(size(runtime))]
    pub vertices: Vec<TerrainChunkVertexInfo>,
}

/**
 * 存储了compute shader计算得出的地形块的索引信息
 */
#[derive(ShaderType)]
pub struct TerrainChunkMeshIndicesVec {
    #[shader(size(runtime))]
    pub indices: Vec<u32>,
}

/**
 * 体素索引对应的顶点索引
 * 是一个映射关系，用于在生成索引时，快速找到对应的顶点
 */
#[derive(ShaderType)]
pub struct TerrainChunkMeshVertexMapVec {
    #[shader(size(runtime))]
    pub vertex_map: Vec<u32>,
}

/**
 * 存储了每个地形块的顶点和索引数量
 */
#[derive(ShaderType)]
pub struct TerrainChunkVerticesIndicesCountVec {
    #[shader(size(runtime))]
    pub vertices_indices_count: Vec<TerrainChunkVerticesIndicesCount>,
}

/// 传递给 GPU compute shader 的地图配置
#[repr(C)]
#[derive(ShaderType, Default, Clone, Copy, Debug, Pod, Zeroable)]
pub struct TerrainMapConfig {
    /// 地形的最大高度
    pub terrain_height: f32,
    /// 一个像素代表的地图大小（米）
    pub pixel_size: f32,
    /// 最小温度（摄氏度）
    pub temperature_min: f32,
    /// 最大温度（摄氏度）
    pub temperature_max: f32,
}

#[cfg(test)]
mod tests {
    use super::TerrainChunkVertexInfo;

    #[test]
    fn test_unpack_u32() {
        let value = 1 + (1 << 8) + (1 << 16) + (1 << 24);
        let result = TerrainChunkVertexInfo::unpack_u32(value);
        assert_eq!(result, [1, 1, 1, 1]);

        let value = 1 + (1 << 8) + (1 << 24);
        let result = TerrainChunkVertexInfo::unpack_u32(value);
        assert_eq!(result, [1, 1, 0, 1]);
    }

    #[test]
    fn test_is_on_border() {
        use bevy::math::{UVec2, UVec4, Vec4};
        // Corner voxel at (0,0,0) - should be on border
        let v = TerrainChunkVertexInfo {
            vertex_location: Vec4::ZERO,
            vertex_normal: Vec4::ZERO,
            vertex_local_coord: UVec4::new(0, 0, 0, 0),
            voxel_biome: UVec2::ZERO,
            voxel_side: UVec2::ZERO,
        };
        assert!(v.is_on_border(16));

        // Interior voxel at (8,8,8) - should NOT be on border
        let v2 = TerrainChunkVertexInfo {
            vertex_local_coord: UVec4::new(8, 8, 8, 0),
            ..v
        };
        assert!(!v2.is_on_border(16));

        // Edge voxel at (15,8,8) - should be on border (15 == 16-1)
        let v3 = TerrainChunkVertexInfo {
            vertex_local_coord: UVec4::new(15, 8, 8, 0),
            ..v
        };
        assert!(v3.is_on_border(16));
    }

    #[test]
    fn test_unpack_u32_zero() {
        assert_eq!(TerrainChunkVertexInfo::unpack_u32(0), [0, 0, 0, 0]);
    }

    #[test]
    fn test_unpack_u32_max() {
        assert_eq!(
            TerrainChunkVertexInfo::unpack_u32(0xFFFFFFFF),
            [255, 255, 255, 255]
        );
    }
}
