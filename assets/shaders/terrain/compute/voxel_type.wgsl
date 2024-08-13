#define_import_path terrain::voxel_type

const U32_MAX: u32 = 0xFFFFFFFFu;
const VOXEL_MATERIAL_AIR: u32 = 0x0u;
const VOXEL_MATERIAL_BLOCK: u32 = 0x1u;

const VOXEL_MATERIAL_NUM: u32 = 2u;
// 不包括air
const VOXEL_MATERIAL_BLOCK_NUM: u32 = 1u;

const VOXEL_MATERIAL_AIR_INDEX: u32 = 0u;

const VOXEL_MATERIAL_TABLE: array<u32, VOXEL_MATERIAL_NUM> = array<u32, VOXEL_MATERIAL_NUM>(
    VOXEL_MATERIAL_AIR,
    VOXEL_MATERIAL_BLOCK,
);


struct TerrainChunkInfo {
    // chunk min location and chunk size
    chunk_min_location_size: vec4<f32>,
    voxel_size: f32,
    voxel_num: u32,
    qef_threshold: f32,
    qef_stddev: f32,
}

struct VoxelEdgeCrossPoint {
    // w is exist or not
    cross_location: vec4<f32>,
    // w is material_index
    normal_material_index: vec4<f32>,
}

struct TerrainChunkVertexInfo {
    location: vec4<f32>,
    normal_materials: vec4<f32>,
}

struct TerrainChunkVerticesIndicesCount {
    vertices_count: atomic<u32>,
    indices_count: atomic<u32>
}
