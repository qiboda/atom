#import terrain::voxel_utils::{get_voxel_vertex_index, get_voxel_index}

#import terrain::main_mesh_bind_group::{
    terrain_chunk_info,
    voxel_vertex_values,
    mesh_vertex_map,
    mesh_vertices_indices_count,
    mesh_indices
}

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
// 考虑考虑是否可以改为dispatch_indirect: 不能，因为并不是每帧都进行计算
// FIXME 没有考虑边缘的indices的情况。
@compute @workgroup_size(4, 4, 4)
fn compute_indices(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    // if (invocation_id.x >= terrain_chunk_info.voxel_num) || (invocation_id.y >= terrain_chunk_info.voxel_num) || (invocation_id.z >= terrain_chunk_info.voxel_num) {
    //     return;
    // }
    var xyz = vec3u(0u, 0u, 0u);
    if invocation_id.x == terrain_chunk_info.voxel_num - 1u {
        xyz.x = 1u;
    }
    if invocation_id.y == terrain_chunk_info.voxel_num - 1u {
        xyz.y = 1u;
    }
    if invocation_id.z == terrain_chunk_info.voxel_num - 1u {
        xyz.z = 1u;
    }

    // if xyz.x + xyz.y + xyz.z > 2u {
    //     return;
    // }

    let voxel_vertex_index_3 = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x + 1, invocation_id.y + 1, invocation_id.z);
    let voxel_vertex_index_5 = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x + 1, invocation_id.y, invocation_id.z + 1);
    let voxel_vertex_index_6 = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y + 1, invocation_id.z + 1);
    let voxel_vertex_index_7 = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x + 1, invocation_id.y + 1, invocation_id.z + 1);

    let voxel_index_0 = get_voxel_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z);
    let voxel_index_1 = get_voxel_index(terrain_chunk_info.voxel_num, invocation_id.x + 1, invocation_id.y, invocation_id.z);
    let voxel_index_2 = get_voxel_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y + 1, invocation_id.z);
    let voxel_index_3 = get_voxel_index(terrain_chunk_info.voxel_num, invocation_id.x + 1, invocation_id.y + 1, invocation_id.z);
    let voxel_index_4 = get_voxel_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z + 1);
    let voxel_index_5 = get_voxel_index(terrain_chunk_info.voxel_num, invocation_id.x + 1, invocation_id.y, invocation_id.z + 1);
    let voxel_index_6 = get_voxel_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y + 1, invocation_id.z + 1);

    // edge is x axis 0 2 4 6
    if xyz.y == 0u && xyz.z == 0u {
        compute_indices_on_axis(voxel_vertex_index_7, voxel_vertex_index_6, voxel_index_0, voxel_index_2, voxel_index_4, voxel_index_6);
    }
    // edge is y axis 0 1 4 5
    if xyz.x == 0u && xyz.z == 0u {
        compute_indices_on_axis(voxel_vertex_index_5, voxel_vertex_index_7, voxel_index_0, voxel_index_1, voxel_index_4, voxel_index_5);
    }
    // edge is z axis 0 1 2 3
    if xyz.x == 0u && xyz.y == 0u {
        compute_indices_on_axis(voxel_vertex_index_7, voxel_vertex_index_3, voxel_index_0, voxel_index_1, voxel_index_2, voxel_index_3);
    }
}

fn compute_indices_on_axis(
    voxel_vertex_index_0: u32, voxel_vertex_index_1: u32,
    voxel_index_0: u32, voxel_index_1: u32, voxel_index_2: u32, voxel_index_3: u32
) {
    let vertex_value_0 = voxel_vertex_values[voxel_vertex_index_0];
    let vertex_value_1 = voxel_vertex_values[voxel_vertex_index_1];
    if (vertex_value_0 >= 0.0 && vertex_value_1 >= 0.0) || (vertex_value_0 < 0.0 && vertex_value_1 < 0.0) {
        return;
    }

    let mesh_vertex_index_0 = mesh_vertex_map[voxel_index_0];
    let mesh_vertex_index_1 = mesh_vertex_map[voxel_index_1];
    let mesh_vertex_index_2 = mesh_vertex_map[voxel_index_2];
    let mesh_vertex_index_3 = mesh_vertex_map[voxel_index_3];

    let mesh_indices_index = atomicAdd(&mesh_vertices_indices_count.indices_count, 6u);

    if vertex_value_0 >= 0.0 {
        mesh_indices[mesh_indices_index] = mesh_vertex_index_0;
        mesh_indices[mesh_indices_index + 1] = mesh_vertex_index_1;
        mesh_indices[mesh_indices_index + 2] = mesh_vertex_index_2;

        mesh_indices[mesh_indices_index + 3] = mesh_vertex_index_1;
        mesh_indices[mesh_indices_index + 4] = mesh_vertex_index_3;
        mesh_indices[mesh_indices_index + 5] = mesh_vertex_index_2;
    } else {
        mesh_indices[mesh_indices_index] = mesh_vertex_index_0;
        mesh_indices[mesh_indices_index + 1] = mesh_vertex_index_2;
        mesh_indices[mesh_indices_index + 2] = mesh_vertex_index_1;

        mesh_indices[mesh_indices_index + 3] = mesh_vertex_index_1;
        mesh_indices[mesh_indices_index + 4] = mesh_vertex_index_2;
        mesh_indices[mesh_indices_index + 5] = mesh_vertex_index_3;
    }
}