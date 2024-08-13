#import noisy::simplex_noise_2d
#import quadric::{Quadric, quadric_default, probabilistic_plane_quadric, quadric_minimizer, quadric_add_quadric, quadric_residual_l2_error}
#import terrain::voxel_type::{TerrainChunkInfo, VoxelEdgeCrossPoint, TerrainChunkVertexInfo, TerrainChunkVerticesIndicesCount, VOXEL_MATERIAL_NUM, VOXEL_MATERIAL_AIR}

#import terrain::voxel_utils::{get_voxel_edge_index, get_voxel_index}
#import terrain::main_mesh_bind_group::{terrain_chunk_info, voxel_cross_points, mesh_vertices, mesh_vertex_map, mesh_vertices_indices_count}

fn compute_cross_point_data(edge_index: u32, qef: ptr<function, Quadric>, location: ptr<function, vec4f>, normal: ptr<function, vec4f>, materials_count: ptr<function, array<vec2u, VOXEL_MATERIAL_NUM>>) {
    let cross_point = voxel_cross_points[edge_index];

    if cross_point.cross_location.w == 0.0 {
        return;
    }

    *location += cross_point.cross_location;

    *normal += cross_point.normal_material_index;

    let quadric = probabilistic_plane_quadric(cross_point.cross_location.xyz, cross_point.normal_material_index.xyz,
        terrain_chunk_info.qef_stddev * terrain_chunk_info.voxel_size, terrain_chunk_info.qef_stddev);
    *qef = quadric_add_quadric(*qef, quadric);

    let material_index = u32(cross_point.normal_material_index.w);
    (*materials_count)[material_index] = (*materials_count)[material_index] + 1u;
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
@compute @workgroup_size(4, 4, 4)
fn compute_vertices(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    // 获取12条边的交点，计算出mesh的顶点的位置
    let edge_0 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z, 0u);
    let edge_1 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z, 1u);
    let edge_2 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z, 2u);

    let edge_3 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x + 1u, invocation_id.y, invocation_id.z, 1u);
    let edge_4 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x + 1u, invocation_id.y, invocation_id.z, 2u);

    let edge_5 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y + 1u, invocation_id.z, 0u);
    let edge_6 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y + 1u, invocation_id.z, 2u);

    let edge_7 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z + 1u, 0u);
    let edge_8 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z + 1u, 1u);

    let edge_9 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x + 1u, invocation_id.y + 1u, invocation_id.z, 2u);
    let edge_10 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x + 1u, invocation_id.y, invocation_id.z + 1u, 1u);
    let edge_11 = get_voxel_edge_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y + 1u, invocation_id.z + 1u, 0u);

    var qef = quadric_default();
    var avg_location = vec4<f32>();
    var avg_normal = vec4<f32>();
    var materials_count = array<vec2u, VOXEL_MATERIAL_NUM>();
    compute_cross_point_data(edge_0, &qef, &avg_location, &avg_normal, &materials_count);
    compute_cross_point_data(edge_1, &qef, &avg_location, &avg_normal, &materials_count);
    compute_cross_point_data(edge_2, &qef, &avg_location, &avg_normal, &materials_count);
    compute_cross_point_data(edge_3, &qef, &avg_location, &avg_normal, &materials_count);
    compute_cross_point_data(edge_4, &qef, &avg_location, &avg_normal, &materials_count);
    compute_cross_point_data(edge_5, &qef, &avg_location, &avg_normal, &materials_count);
    compute_cross_point_data(edge_6, &qef, &avg_location, &avg_normal, &materials_count);
    compute_cross_point_data(edge_7, &qef, &avg_location, &avg_normal, &materials_count);
    compute_cross_point_data(edge_8, &qef, &avg_location, &avg_normal, &materials_count);
    compute_cross_point_data(edge_9, &qef, &avg_location, &avg_normal, &materials_count);
    compute_cross_point_data(edge_10, &qef, &avg_location, &avg_normal, &materials_count);
    compute_cross_point_data(edge_11, &qef, &avg_location, &avg_normal, &materials_count);

    let count = avg_location.w;
    if count <= 0.0 {
        return;
    }

    var qef_location = quadric_minimizer(qef);
    if quadric_residual_l2_error(qef, qef_location) < terrain_chunk_info.qef_threshold {
        avg_location = vec4f(qef_location, 1.0);
    } else {
        avg_location = avg_location / count;
    }

    avg_normal.w = 0.0;
    avg_normal = normalize(avg_normal);

    var max_count = 0u;
    var material = VOXEL_MATERIAL_AIR;
    for (var i = 0u; i < VOXEL_MATERIAL_NUM; i++) {
        if materials_count[i].y > max_count {
            max_count = materials_count[i].y;
            material = materials_count[i].x;
        }
    }

    let vertex_index = atomicAdd(&mesh_vertices_indices_count.vertices_count, 1u);

    mesh_vertices[vertex_index].location = avg_location;
    mesh_vertices[vertex_index].normal_materials = avg_normal;
    mesh_vertices[vertex_index].normal_materials.w = f32(material);
    let voxel_index = get_voxel_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z);
    mesh_vertex_map[voxel_index] = vertex_index;
}