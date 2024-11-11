#import noisy::simplex_noise_2d
#import quadric::{Quadric, quadric_default, probabilistic_plane_quadric, quadric_minimizer, quadric_add_quadric, quadric_residual_l2_error}
#import terrain::voxel_type::{TerrainChunkInfo, VoxelEdgeCrossPoint, TerrainChunkVertexInfo, TerrainChunkVerticesIndicesCount, VOXEL_MATERIAL_NUM, VOXEL_MATERIAL_AIR, VOXEL_MATERIAL_TABLE}

#import terrain::voxel_utils::{get_voxel_edge_index, get_voxel_index, get_voxel_vertex_index, get_voxel_material_type}
#import terrain::main_mesh_bind_group::{terrain_chunk_info, voxel_cross_points, mesh_vertices, mesh_vertex_map, mesh_vertices_indices_count, voxel_vertex_values}
#import terrain::density_field::get_biome_type_by_location

#import math::pack::pack4xU8

fn compute_cross_point_data(edge_index: u32, qef: ptr<function, Quadric>, location: ptr<function, vec4f>, normal: ptr<function, vec4f>) {
    let cross_point = voxel_cross_points[edge_index];

    if cross_point.cross_location.w == 0.0 {
        return;
    }

    *location += cross_point.cross_location;

    *normal += cross_point.normal_material_index;

    let quadric = probabilistic_plane_quadric(cross_point.cross_location.xyz, cross_point.normal_material_index.xyz,
        terrain_chunk_info.qef_stddev * terrain_chunk_info.voxel_size, terrain_chunk_info.qef_stddev);
    *qef = quadric_add_quadric(*qef, quadric);
}

fn is_in_aabb(location: vec3f, min_location: vec3f, max_location: vec3f) -> bool {
    return all(min_location < location) && all(location < max_location);
}

fn clamp_aabb(location: vec3f, min_location: vec3f, max_location: vec3f) -> vec3f {
    return min(max(location, min_location), max_location);
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

    let voxel_min_location = terrain_chunk_info.chunk_min_location_size.xyz + vec3<f32>(
        f32(invocation_id.x),
        f32(invocation_id.y),
        f32(invocation_id.z),
    ) * terrain_chunk_info.voxel_size;

    let voxel_max_location = terrain_chunk_info.chunk_min_location_size.xyz + vec3<f32>(
        f32(invocation_id.x + 1),
        f32(invocation_id.y + 1),
        f32(invocation_id.z + 1),
    ) * terrain_chunk_info.voxel_size;

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
    compute_cross_point_data(edge_0, &qef, &avg_location, &avg_normal);
    compute_cross_point_data(edge_1, &qef, &avg_location, &avg_normal);
    compute_cross_point_data(edge_2, &qef, &avg_location, &avg_normal);
    compute_cross_point_data(edge_3, &qef, &avg_location, &avg_normal);
    compute_cross_point_data(edge_4, &qef, &avg_location, &avg_normal);
    compute_cross_point_data(edge_5, &qef, &avg_location, &avg_normal);
    compute_cross_point_data(edge_6, &qef, &avg_location, &avg_normal);
    compute_cross_point_data(edge_7, &qef, &avg_location, &avg_normal);
    compute_cross_point_data(edge_8, &qef, &avg_location, &avg_normal);
    compute_cross_point_data(edge_9, &qef, &avg_location, &avg_normal);
    compute_cross_point_data(edge_10, &qef, &avg_location, &avg_normal);
    compute_cross_point_data(edge_11, &qef, &avg_location, &avg_normal);

    let count = avg_location.w;
    if count <= 0.0 {
        return;
    }

    var qef_location = quadric_minimizer(qef);
    if quadric_residual_l2_error(qef, qef_location) < terrain_chunk_info.qef_threshold {
        if is_in_aabb(qef_location, voxel_min_location, voxel_max_location) {
            avg_location = vec4f(qef_location, 1.0);
        } else {
            avg_location = vec4f(clamp_aabb(qef_location, voxel_min_location + vec3f(0.01, 0.01, 0.01), voxel_max_location - vec3f(0.01, 0.01, 0.01)), 1.0);
        }
    } else {
        avg_location = avg_location / count;
    }

    avg_normal.w = 0.0;
    avg_normal = normalize(avg_normal);

    let vertex_index = atomicAdd(&mesh_vertices_indices_count.vertices_count, 1u);

    mesh_vertices[vertex_index].location = avg_location;
    mesh_vertices[vertex_index].normal = avg_normal;
    mesh_vertices[vertex_index].local_coord = vec4u(invocation_id, 0u);

    let value_index_0 = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z);
    let value_index_1 = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x + 1u, invocation_id.y, invocation_id.z);
    let value_index_2 = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y + 1u, invocation_id.z);
    let value_index_3 = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x + 1u, invocation_id.y + 1u, invocation_id.z);
    let value_index_4 = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z + 1u);
    let value_index_5 = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x + 1u, invocation_id.y, invocation_id.z + 1u);
    let value_index_6 = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y + 1u, invocation_id.z + 1u);
    let value_index_7 = get_voxel_vertex_index(terrain_chunk_info.voxel_num, invocation_id.x + 1u, invocation_id.y + 1u, invocation_id.z + 1u);
    // >= 0.0 is outside, < 0.0 is inside
    let v0 = u32(voxel_vertex_values[value_index_0] >= 0.0);
    let v1 = u32(voxel_vertex_values[value_index_1] >= 0.0);
    let v2 = u32(voxel_vertex_values[value_index_2] >= 0.0);
    let v3 = u32(voxel_vertex_values[value_index_3] >= 0.0);
    let v4 = u32(voxel_vertex_values[value_index_4] >= 0.0);
    let v5 = u32(voxel_vertex_values[value_index_5] >= 0.0);
    let v6 = u32(voxel_vertex_values[value_index_6] >= 0.0);
    let v7 = u32(voxel_vertex_values[value_index_7] >= 0.0);
    let side_0 = pack4xU8(vec4u(v0, v1, v2, v3));
    let side_1 = pack4xU8(vec4u(v4, v5, v6, v7));
    mesh_vertices[vertex_index].voxel_side = vec2u(side_0, side_1);

    let voxel_biome_0 = get_biome_type_by_location(voxel_min_location + vec3f(0.0, 0.0, 0.0));
    let voxel_biome_1 = get_biome_type_by_location(voxel_min_location + vec3f(terrain_chunk_info.voxel_size, 0.0, 0.0));
    let voxel_biome_2 = get_biome_type_by_location(voxel_min_location + vec3f(0.0, terrain_chunk_info.voxel_size, 0.0));
    let voxel_biome_3 = get_biome_type_by_location(voxel_min_location + vec3f(terrain_chunk_info.voxel_size, terrain_chunk_info.voxel_size, 0.0));
    let voxel_biome_4 = get_biome_type_by_location(voxel_min_location + vec3f(0.0, 0.0, terrain_chunk_info.voxel_size));
    let voxel_biome_5 = get_biome_type_by_location(voxel_min_location + vec3f(terrain_chunk_info.voxel_size, 0.0, terrain_chunk_info.voxel_size));
    let voxel_biome_6 = get_biome_type_by_location(voxel_min_location + vec3f(0.0, terrain_chunk_info.voxel_size, terrain_chunk_info.voxel_size));
    let voxel_biome_7 = get_biome_type_by_location(voxel_min_location + vec3f(terrain_chunk_info.voxel_size, terrain_chunk_info.voxel_size, terrain_chunk_info.voxel_size));

    let voxel_biome_00 = pack4xU8(vec4u(voxel_biome_0, voxel_biome_1, voxel_biome_2, voxel_biome_3));
    let voxel_biome_01 = pack4xU8(vec4u(voxel_biome_4, voxel_biome_5, voxel_biome_6, voxel_biome_7));
    mesh_vertices[vertex_index].voxel_biome = vec2u(voxel_biome_00, voxel_biome_01);

    let voxel_index = get_voxel_index(terrain_chunk_info.voxel_num, invocation_id.x, invocation_id.y, invocation_id.z);
    mesh_vertex_map[voxel_index] = vertex_index;
}