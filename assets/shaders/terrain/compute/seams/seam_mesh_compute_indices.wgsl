/// x 方向，
#import quadric::{Quadric, quadric_default, probabilistic_plane_quadric, quadric_minimizer, quadric_add_quadric, quadric_residual_l2_error}
#import terrain::voxel_type::{TerrainChunkInfo, VoxelEdgeCrossPoint, VOXEL_MATERIAL_NUM, VOXEL_MATERIAL_AIR}
#import terrain::voxel_utils::{get_voxel_edge_index, get_voxel_index, }
#import terrain::seam_utils::{axis_base_offset, coord_convert_fns, get_voxel_internal_vertex_index, VOXEL_VERTEX_OFFSETS}
#import terrain::density_field::get_terrain_noise 
#import terrain::voxel_type::{U32_MAX}

@group(0) @binding(0)
var<uniform> terrain_chunk_info: TerrainChunkInfo;

@group(0) @binding(1)
var<storage, read> mesh_vertex_map: array<u32>;

@group(0) @binding(2)
var<storage, read_write> mesh_indices_data: array<u32>;

@group(0) @binding(3)
var<storage, read_write> mesh_indices_num: atomic<u32>;

// 存储了mesh顶点的索引, 该结构体在array中，array的索引是体素的索引。
//
// Vertex and Edge Index Map
//
//       2-------1------3
//      /.             /|
//     10.           11 |
//    /  4           /  5
//   /   .          /   |     ^ Y
//  6-------3------7    |     |
//  |    0 . . 0 . |. . 1     --> X
//  |   .          |   /     /
//  6  8           7  9     / z
//  | .            | /     |/
//  |.             |/
//  4-------2------5
//
// 计算seam的三个面之一，包括了外部chunk一部分的体素。
// 33 * 33 * 2 or 65 * 65 * 2

// chunk的z axis方向的缝隙voxel。
@compute @workgroup_size(1, 1, 2)
fn compute_indices_z_axis(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    if (invocation_id.x > terrain_chunk_info.voxel_num) || (invocation_id.y > terrain_chunk_info.voxel_num) {
        return;
    }

    var xyz = vec3u(0u, 0u, 0u);
    xyz.x = select(0u, 1u, invocation_id.x == terrain_chunk_info.voxel_num);
    xyz.y = select(0u, 1u, invocation_id.y == terrain_chunk_info.voxel_num);
    xyz.z = select(0u, 1u, invocation_id.z == 1u);

    let seam_voxel_size = vec3u(terrain_chunk_info.voxel_num + 1, terrain_chunk_info.voxel_num + 1, 2);

    // 缝隙chunk的最小位置
    let chunk_base_offset = vec3u(0u, 0u, 1u) * (terrain_chunk_info.voxel_num - 1);
    // 整个chunk的坐标
    let chunk_coord = chunk_base_offset + invocation_id;
    // 当前体素的最小位置
    let seam_voxel_min_location = terrain_chunk_info.chunk_min_location_size.xyz + vec3f(chunk_coord) * terrain_chunk_info.voxel_size;

    let vertex_index_0 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x, invocation_id.y, invocation_id.z);
    let vertex_index_1 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x + 1u, invocation_id.y, invocation_id.z);
    let vertex_index_2 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x, invocation_id.y + 1u, invocation_id.z);
    let vertex_index_3 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x + 1u, invocation_id.y + 1u, invocation_id.z);
    let vertex_index_4 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x, invocation_id.y, invocation_id.z + 1u);
    let vertex_index_5 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x + 1u, invocation_id.y, invocation_id.z + 1u);
    let vertex_index_6 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x, invocation_id.y + 1u, invocation_id.z + 1u);

    let voxel_vertex_value_3 = get_terrain_noise(seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[3]);
    let voxel_vertex_value_5 = get_terrain_noise(seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[5]);
    let voxel_vertex_value_6 = get_terrain_noise(seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[6]);
    let voxel_vertex_value_7 = get_terrain_noise(seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[7]);

    // x axis 0 2 4 6
    if xyz.y == 0u && xyz.z == 0u {
        compute_indices_on_axis(voxel_vertex_value_7, voxel_vertex_value_6, vertex_index_0, vertex_index_2, vertex_index_4, vertex_index_6);
    }
    // y axis 0 1 4 5
    if xyz.x == 0u && xyz.z == 0u {
        compute_indices_on_axis(voxel_vertex_value_5, voxel_vertex_value_7, vertex_index_0, vertex_index_1, vertex_index_4, vertex_index_5);
    }
    // z axis 0 1 2 3
    if xyz.x == 0u && xyz.y == 0u {
        compute_indices_on_axis(voxel_vertex_value_7, voxel_vertex_value_3, vertex_index_0, vertex_index_1, vertex_index_2, vertex_index_3);
    }
}
// chunk的x axis方向的缝隙voxel。
@compute @workgroup_size(1, 2, 1)
fn compute_indices_y_axis(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    if (invocation_id.x > terrain_chunk_info.voxel_num) || (invocation_id.z > terrain_chunk_info.voxel_num) {
        return;
    }

    let seam_voxel_size = vec3u(terrain_chunk_info.voxel_num + 1, 2, terrain_chunk_info.voxel_num + 1);

    var xyz = vec3u(0u, 0u, 0u);
    xyz.x = select(0u, 1u, invocation_id.x == terrain_chunk_info.voxel_num);
    xyz.y = select(0u, 1u, invocation_id.y == 1u);
    xyz.z = select(0u, 1u, invocation_id.z == terrain_chunk_info.voxel_num);

    // 缝隙chunk的最小位置
    let chunk_base_offset = vec3u(0u, 1u, 0u) * (terrain_chunk_info.voxel_num - 1);
    // 整个chunk的坐标
    let chunk_coord = chunk_base_offset + invocation_id;
    // 当前体素的最小位置
    let seam_voxel_min_location = terrain_chunk_info.chunk_min_location_size.xyz + vec3f(chunk_coord) * terrain_chunk_info.voxel_size;

    let vertex_index_0 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x, invocation_id.y, invocation_id.z);
    let vertex_index_1 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x + 1u, invocation_id.y, invocation_id.z);
    let vertex_index_2 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x, invocation_id.y + 1u, invocation_id.z);
    let vertex_index_3 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x + 1u, invocation_id.y + 1u, invocation_id.z);
    let vertex_index_4 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x, invocation_id.y, invocation_id.z + 1u);
    let vertex_index_5 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x + 1u, invocation_id.y, invocation_id.z + 1u);
    let vertex_index_6 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x, invocation_id.y + 1u, invocation_id.z + 1u);

    let voxel_vertex_value_3 = get_terrain_noise(seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[3]);
    let voxel_vertex_value_5 = get_terrain_noise(seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[5]);
    let voxel_vertex_value_6 = get_terrain_noise(seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[6]);
    let voxel_vertex_value_7 = get_terrain_noise(seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[7]);

    // x axis 0 2 4 6
    if xyz.y == 0u && xyz.z == 0u {
        compute_indices_on_axis(voxel_vertex_value_7, voxel_vertex_value_6, vertex_index_0, vertex_index_2, vertex_index_4, vertex_index_6);
    }
    // y axis 0 1 4 5
    if xyz.x == 0u && xyz.z == 0u {
        compute_indices_on_axis(voxel_vertex_value_5, voxel_vertex_value_7, vertex_index_0, vertex_index_1, vertex_index_4, vertex_index_5);
    }
    // z axis 0 1 2 3
    if xyz.x == 0u && xyz.y == 0u {
        compute_indices_on_axis(voxel_vertex_value_7, voxel_vertex_value_3, vertex_index_0, vertex_index_1, vertex_index_2, vertex_index_3);
    }
}

// chunk的x axis方向的缝隙voxel。
@compute @workgroup_size(2, 1, 1)
fn compute_indices_x_axis(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    if (invocation_id.y > terrain_chunk_info.voxel_num) || (invocation_id.z > terrain_chunk_info.voxel_num) {
        return;
    }

    var xyz = vec3u(0u, 0u, 0u);
    xyz.x = select(0u, 1u, invocation_id.x == 1u);
    xyz.y = select(0u, 1u, invocation_id.y == terrain_chunk_info.voxel_num);
    xyz.z = select(0u, 1u, invocation_id.z == terrain_chunk_info.voxel_num);

    let seam_voxel_size = vec3u(2, terrain_chunk_info.voxel_num + 1, terrain_chunk_info.voxel_num + 1);

    // 缝隙chunk的最小位置
    let chunk_base_offset = vec3u(1u, 0u, 0u) * (terrain_chunk_info.voxel_num - 1);
    // 整个chunk的坐标
    let chunk_coord = chunk_base_offset + invocation_id;
    // 当前体素的最小位置
    let seam_voxel_min_location = terrain_chunk_info.chunk_min_location_size.xyz + vec3f(chunk_coord) * terrain_chunk_info.voxel_size;

    let vertex_index_0 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x, invocation_id.y, invocation_id.z);
    let vertex_index_1 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x + 1u, invocation_id.y, invocation_id.z);
    let vertex_index_2 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x, invocation_id.y + 1u, invocation_id.z);
    let vertex_index_3 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x + 1u, invocation_id.y + 1u, invocation_id.z);
    let vertex_index_4 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x, invocation_id.y, invocation_id.z + 1u);
    let vertex_index_5 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x + 1u, invocation_id.y, invocation_id.z + 1u);
    let vertex_index_6 = get_voxel_internal_vertex_index(seam_voxel_size, invocation_id.x, invocation_id.y + 1u, invocation_id.z + 1u);

    let voxel_vertex_value_3 = get_terrain_noise(seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[3]);
    let voxel_vertex_value_5 = get_terrain_noise(seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[5]);
    let voxel_vertex_value_6 = get_terrain_noise(seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[6]);
    let voxel_vertex_value_7 = get_terrain_noise(seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[7]);

    // x axis 0 2 4 6
    if xyz.y == 0u && xyz.z == 0u {
        compute_indices_on_axis(voxel_vertex_value_7, voxel_vertex_value_6, vertex_index_0, vertex_index_2, vertex_index_4, vertex_index_6);
    }
    // y axis 0 1 4 5
    if xyz.x == 0u && xyz.z == 0u {
        compute_indices_on_axis(voxel_vertex_value_5, voxel_vertex_value_7, vertex_index_0, vertex_index_1, vertex_index_4, vertex_index_5);
    }
    // z axis 0 1 2 3
    if xyz.x == 0u && xyz.y == 0u {
        compute_indices_on_axis(voxel_vertex_value_7, voxel_vertex_value_3, vertex_index_0, vertex_index_1, vertex_index_2, vertex_index_3);
    }
}

fn compute_indices_on_axis(
    vertex_value_0: f32, vertex_value_1: f32,
    voxel_index_0: u32, voxel_index_1: u32, voxel_index_2: u32, voxel_index_3: u32
) {
    if (vertex_value_0 >= 0.0 && vertex_value_1 >= 0.0) || (vertex_value_0 < 0.0 && vertex_value_1 < 0.0) {
        return;
    }

    let mesh_vertex_index_0 = mesh_vertex_map[voxel_index_0];
    let mesh_vertex_index_1 = mesh_vertex_map[voxel_index_1];
    let mesh_vertex_index_2 = mesh_vertex_map[voxel_index_2];
    let mesh_vertex_index_3 = mesh_vertex_map[voxel_index_3];

    // 调整2 3的位置，使得，任意从小到大排列总是逆时针。
    var mesh_vertex_index_array = vec4u(U32_MAX, U32_MAX, U32_MAX, U32_MAX);
    let index_0 = set_vertex_index_array(&mesh_vertex_index_array, mesh_vertex_index_0, 0u);
    let index_1 = set_vertex_index_array(&mesh_vertex_index_array, mesh_vertex_index_1, index_0);
    let index_2 = set_vertex_index_array(&mesh_vertex_index_array, mesh_vertex_index_3, index_1);
    let index_3 = set_vertex_index_array(&mesh_vertex_index_array, mesh_vertex_index_2, index_2);

    if index_3 == 3 {
        let mesh_indices_index = atomicAdd(&mesh_indices_num, 3u);

        if vertex_value_0 >= 0.0 {
            mesh_indices_data[mesh_indices_index] = mesh_vertex_index_array[0];
            mesh_indices_data[mesh_indices_index + 1] = mesh_vertex_index_array[1];
            mesh_indices_data[mesh_indices_index + 2] = mesh_vertex_index_array[2];
        } else {
            mesh_indices_data[mesh_indices_index] = mesh_vertex_index_array[0] ;
            mesh_indices_data[mesh_indices_index + 1] = mesh_vertex_index_array[2] ;
            mesh_indices_data[mesh_indices_index + 2] = mesh_vertex_index_array[1] ;
        }
    } else if index_3 == 4 {
        let mesh_indices_index = atomicAdd(&mesh_indices_num, 6u);

        if vertex_value_0 >= 0.0 {
            mesh_indices_data[mesh_indices_index] = mesh_vertex_index_array[0];
            mesh_indices_data[mesh_indices_index + 1] = mesh_vertex_index_array[1];
            mesh_indices_data[mesh_indices_index + 2] = mesh_vertex_index_array[2];

            mesh_indices_data[mesh_indices_index + 3] = mesh_vertex_index_array[0];
            mesh_indices_data[mesh_indices_index + 4] = mesh_vertex_index_array[2];
            mesh_indices_data[mesh_indices_index + 5] = mesh_vertex_index_array[3];
        } else {
            mesh_indices_data[mesh_indices_index] = mesh_vertex_index_array[0];
            mesh_indices_data[mesh_indices_index + 1] = mesh_vertex_index_array[2];
            mesh_indices_data[mesh_indices_index + 2] = mesh_vertex_index_array[1];

            mesh_indices_data[mesh_indices_index + 3] = mesh_vertex_index_array[0];
            mesh_indices_data[mesh_indices_index + 4] = mesh_vertex_index_array[3];
            mesh_indices_data[mesh_indices_index + 5] = mesh_vertex_index_array[2];
        }
    }
}

fn set_vertex_index_array(index_array: ptr<function, vec4u>, value: u32, index: u32) -> u32 {
    if any(vec4((*index_array)[0] == value, (*index_array)[1] == value, (*index_array)[2] == value, (*index_array)[3] == value)) {
        return index;
    }

    (*index_array)[index] = select((*index_array)[index], value, (*index_array)[index] == U32_MAX && value != U32_MAX);
    return index + u32(select(0, 1, (*index_array)[index] == value));
}
