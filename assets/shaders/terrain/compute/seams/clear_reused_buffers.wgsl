#import terrain::voxel_type::U32_MAX
#import terrain::voxel_type::{TerrainChunkInfo}
#import terrain::seam_utils::{get_voxel_internal_vertex_index}

@group(0) @binding(0)
var<uniform> terrain_chunk_info: TerrainChunkInfo;

@group(0) @binding(1)
var<storage, read_write> mesh_vertex_map: array<u32>;

@compute @workgroup_size(1, 1, 2)
fn clear_reused_buffers(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    // if (invocation_id.x > terrain_chunk_info.voxel_num) || (invocation_id.y > terrain_chunk_info.voxel_num) {
    //     return;
    // }

    // let seam_chunk_size = vec3u(2, terrain_chunk_info.voxel_num + 2, terrain_chunk_info.voxel_num + 2);

    // let voxel_index = get_voxel_internal_vertex_index(
    //     seam_chunk_size,
    //     invocation_id.x,
    //     invocation_id.y,
    //     invocation_id.z,
    // );
    // mesh_vertex_map[voxel_index] = U32_MAX;
}