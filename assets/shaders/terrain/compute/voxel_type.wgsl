#define_import_path terrain::voxel_type

struct TerrainChunkInfo {
    // chunk min location and terrain size
    chunk_min_location_size: vec4<f32>,
    voxel_size: f32,
    voxel_num: u32,
    qef_threshold: f32,
    qef_stddev: f32,
}

struct VoxelEdgeCrossPoint {
    // w is exist or not
    cross_location: vec4<f32>,
    // w is 没有用
    normal_material_index: vec4<f32>,
}

struct TerrainChunkVertexInfo {
    location: vec4<f32>,
    normal: vec4<f32>,
    local_coord: vec4u,
    voxel_biome: vec2u,
    voxel_side: vec2u,
}

struct TerrainChunkVerticesIndicesCount {
    vertices_count: atomic<u32>,
    indices_count: atomic<u32>
}

struct TerrainMapConfig {
    // 地形的最大高度
    terrain_height: f32,
    // 一个像素代表的地图大小
    pixel_size: f32,
    // 最小温度
    temperature_min: f32,
    // 最大温度
    temperature_max: f32,
}
