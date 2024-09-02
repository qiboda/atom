use bevy::{
    math::{UVec2, UVec4, Vec3, Vec4},
    render::render_resource::ShaderType,
};
use bytemuck::{Pod, Zeroable};

use crate::{
    isosurface::{select_voxel_biome, IsosurfaceSide},
    map::topography::MapFlatTerrainType,
};

#[derive(ShaderType, Default, Clone, Copy, Debug)]
pub struct VoxelEdgeCrossPoint {
    // w is exist or not
    pub cross_pos: Vec4,
    // xyz is normal, w is material_index
    pub normal_material_index: Vec4,
}

#[derive(ShaderType, Default)]
pub struct TerrainChunkInfo {
    // aabb的min 和 w作为chunk的size
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

    pub fn get_vertex_biome(&self) -> MapFlatTerrainType {
        let biomes = self.get_voxel_biome();
        select_voxel_biome(biomes)
    }

    pub fn get_voxel_biome(&self) -> [MapFlatTerrainType; 8] {
        let x = TerrainChunkVertexInfo::unpack_u32(self.voxel_biome.x)
            .map(|x| MapFlatTerrainType::from_repr(x as usize).unwrap());
        let y = TerrainChunkVertexInfo::unpack_u32(self.voxel_biome.y)
            .map(|x| MapFlatTerrainType::from_repr(x as usize).unwrap());
        [x[0], x[1], x[2], x[3], y[0], y[1], y[2], y[3]]
    }
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

#[derive(ShaderType)]
pub struct VoxelVertexValueVec {
    #[size(runtime)]
    pub values: Vec<f32>,
}

#[derive(ShaderType)]
pub struct VoxelEdgeCrossPointVec {
    #[size(runtime)]
    pub cross_points: Vec<VoxelEdgeCrossPoint>,
}

#[derive(ShaderType)]
pub struct TerrainChunkMeshVertexInfoVec {
    #[size(runtime)]
    pub vertices: Vec<TerrainChunkVertexInfo>,
}

#[derive(ShaderType)]
pub struct TerrainChunkMeshIndicesVec {
    #[size(runtime)]
    pub indices: Vec<u32>,
}

#[derive(ShaderType)]
pub struct TerrainChunkMeshVertexMapVec {
    #[size(runtime)]
    pub vertex_map: Vec<u32>,
}

#[derive(ShaderType)]
pub struct TerrainChunkVerticesIndicesCountVec {
    #[size(runtime)]
    pub vertices_indices_count: Vec<TerrainChunkVerticesIndicesCount>,
}

#[cfg(test)]
mod tests {
    use crate::isosurface::dc::gpu_dc::buffer_type::TerrainChunkVertexInfo;

    #[test]
    fn test_unpack_u32() {
        let value = 1 + (1 << 8) + (1 << 16) + (1 << 24);
        let result = TerrainChunkVertexInfo::unpack_u32(value);
        assert_eq!(result, [1, 1, 1, 1]);

        let value = 1 + (1 << 8) + (1 << 24);
        let result = TerrainChunkVertexInfo::unpack_u32(value);
        assert_eq!(result, [1, 1, 0, 1]);
    }
}
