/// 计算体素顶点的密度值以及
#import terrain::voxel_type::{TerrainChunkInfo, VoxelEdgeCrossPoint, VOXEL_MATERIAL_AIR_INDEX}
#import terrain::voxel_utils::{get_voxel_vertex_index, get_voxel_edge_index, get_voxel_material_type_index, central_gradient}
#import terrain::density_field::get_terrain_noise
#import terrain::main_mesh_bind_group:: {
    terrain_chunk_info,
    voxel_vertex_values,
    voxel_cross_points
}

fn estimate_edge_cross_point(
    left_vertex_index: u32,
    right_vertex_index: u32,
    left_vertex_location: vec3<f32>,
    right_vertex_location: vec3<f32>,
    edge_index: u32
) {
    let s1 = voxel_vertex_values[left_vertex_index];
    let s2 = voxel_vertex_values[right_vertex_index];
    if (s1 < 0.0 && s2 >= 0.0) || (s1 >= 0.0 && s2 < 0.0) {

        var dir = 1.0;
        if s2 > s1 {
            dir = 1.0;
        } else {
            dir = -1.0;
        }

        var cross_pos = left_vertex_location + (right_vertex_location - left_vertex_location) * 0.5;
        var step = (right_vertex_location - left_vertex_location) / 4.0;
        var cross_value = get_terrain_noise(cross_pos);
        for (var j = 0u; j < 8u; j++) {
            if cross_value == 0.0 {
                break;
            } else {
                var offset_dir = dir;
                if cross_value < 0.0 {
                    offset_dir = dir ;
                } else {
                    offset_dir = -dir;
                };
                cross_pos += offset_dir * step;
                cross_value = get_terrain_noise(cross_pos);
                step /= 2.0;
            }
        }

        // 因为有一个必为Air，不需要记录
        let s1_material_type_index = get_voxel_material_type_index(s1);
        let s2_material_type_index = get_voxel_material_type_index(s2);
        let material_index = max(s1_material_type_index, s2_material_type_index);

        let normal = central_gradient(cross_pos, terrain_chunk_info.voxel_size);
        voxel_cross_points[edge_index] = VoxelEdgeCrossPoint(vec4f(cross_pos, 1.0), vec4f(normal, f32(material_index)));
    } else {
        voxel_cross_points[edge_index] = VoxelEdgeCrossPoint(vec4f(0.0, 0.0, 0.0, 0.0), vec4f(0.0, 0.0, 0.0, f32(VOXEL_MATERIAL_AIR_INDEX)));
    }
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
// voxel one vertex to three edge
@compute @workgroup_size(4, 4, 4)
fn compute_voxel_cross_points(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    // 顶点比体素数量多1
    if (invocation_id.x > terrain_chunk_info.voxel_num) || (invocation_id.y > terrain_chunk_info.voxel_num) || (invocation_id.z > terrain_chunk_info.voxel_num) {
        return;
    }

    let voxel_vertex_location = terrain_chunk_info.chunk_min_location_size.xyz + vec3<f32>(
        f32(invocation_id.x),
        f32(invocation_id.y),
        f32(invocation_id.z),
    ) * terrain_chunk_info.voxel_size;

    let vertex_index = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z);

    var xyz = vec3u(0u, 0u, 0u);
    if invocation_id.x == terrain_chunk_info.voxel_num {
        xyz.x = 1u;
    }
    if invocation_id.y == terrain_chunk_info.voxel_num {
        xyz.y = 1u;
    }
    if invocation_id.z == terrain_chunk_info.voxel_num {
        xyz.z = 1u;
    }

    if xyz.x == 0u {
        let edge_index_0 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z, 0u);
        let vertex_index_x = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x + 1, invocation_id.y, invocation_id.z);
        let voxel_vertex_location_x = voxel_vertex_location + vec3<f32>(terrain_chunk_info.voxel_size, 0.0, 0.0);
        estimate_edge_cross_point(vertex_index, vertex_index_x, voxel_vertex_location, voxel_vertex_location_x, edge_index_0);
    }

    if xyz.y == 0u {
        let edge_index_1 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z, 1u);
        let voxel_vertex_location_y = voxel_vertex_location + vec3<f32>(0.0, terrain_chunk_info.voxel_size, 0.0);
        let vertex_index_y = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y + 1, invocation_id.z);
        estimate_edge_cross_point(vertex_index, vertex_index_y, voxel_vertex_location, voxel_vertex_location_y, edge_index_1);
    }

    if xyz.z == 0u {
        let edge_index_2 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z, 2u);
        let voxel_vertex_location_z = voxel_vertex_location + vec3<f32>(0.0, 0.0, terrain_chunk_info.voxel_size);
        let vertex_index_z = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z + 1);
        estimate_edge_cross_point(vertex_index, vertex_index_z, voxel_vertex_location, voxel_vertex_location_z, edge_index_2);
    }
}