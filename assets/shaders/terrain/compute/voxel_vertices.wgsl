/// 计算体素顶点的密度值以及
#import terrain::voxel_type::{TerrainChunkInfo}
#import terrain::density_field::get_terrain_noise
#import terrain::voxel_utils::get_voxel_vertex_index

@group(0) @binding(0)
var<uniform> terrain_chunk_info: TerrainChunkInfo;

@group(0) @binding(1)
var<storage, read_write> voxel_vertex_values: array<f32>;

@compute @workgroup_size(4, 4, 4)
fn compute_voxel_vertices(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    // 顶点比体素数量多1
    if (invocation_id.x > terrain_chunk_info.voxel_num) || (invocation_id.y > terrain_chunk_info.voxel_num) || (invocation_id.z > terrain_chunk_info.voxel_num) {
        return;
    }
    let voxel_min_location = terrain_chunk_info.chunk_min_location_size.xyz + vec3<f32>(
        f32(invocation_id.x),
        f32(invocation_id.y),
        f32(invocation_id.z),
    ) * terrain_chunk_info.voxel_size;

    let index = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z);
    let value = get_terrain_noise(voxel_min_location);
    voxel_vertex_values[index] = value;
}