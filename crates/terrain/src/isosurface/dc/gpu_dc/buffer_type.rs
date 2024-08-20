use bevy::{
    math::{UVec4, Vec3, Vec4},
    render::render_resource::ShaderType,
};
use bytemuck::{Pod, Zeroable};
use strum::EnumCount;

use crate::{isosurface::voxel::VoxelMaterialType, tables::SubNodeIndex};

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
    pub vertex_normal_materials: Vec4,
    pub vertex_local_coord: UVec4,
    pub voxel_materials_0: UVec4,
    pub voxel_materials_1: UVec4,
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

    pub fn get_material(&self) -> VoxelMaterialType {
        VoxelMaterialType::from(self.vertex_normal_materials.w as u32)
    }

    pub fn get_voxel_materials(&self) -> [VoxelMaterialType; SubNodeIndex::COUNT] {
        [
            VoxelMaterialType::from(self.voxel_materials_0.x),
            VoxelMaterialType::from(self.voxel_materials_0.y),
            VoxelMaterialType::from(self.voxel_materials_0.z),
            VoxelMaterialType::from(self.voxel_materials_0.w),
            VoxelMaterialType::from(self.voxel_materials_1.x),
            VoxelMaterialType::from(self.voxel_materials_1.y),
            VoxelMaterialType::from(self.voxel_materials_1.z),
            VoxelMaterialType::from(self.voxel_materials_1.w),
        ]
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
