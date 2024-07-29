/// x 方向，
#import noisy::simplex_noise_2d
#import quadric::{Quadric, quadric_default, probabilistic_plane_quadric, quadric_minimizer, quadric_add_quadric, quadric_residual_l2_error}
#import terrain::voxel_type::{TerrainChunkInfo, VoxelEdgeCrossPoint, VOXEL_MATERIAL_NUM, VOXEL_MATERIAL_AIR, VOXEL_MATERIAL_AIR_INDEX, U32_MAX}
#import terrain::voxel_utils::{get_voxel_edge_index, get_voxel_index, get_voxel_material_type_index, central_gradient}
#import terrain::seam_utils::{axis_base_offset, coord_convert_fns, get_voxel_internal_vertex_index, EDGE_VERTEX_PAIRS, VOXEL_VERTEX_OFFSETS}
#import terrain::density_field::get_terrain_noise


@group(0) @binding(0)
var<uniform> terrain_chunk_info: TerrainChunkInfo;

@group(0) @binding(1)
var<uniform> terrain_chunks_lod: array<vec4<u32>, 16>;

@group(0) @binding(2)
var<storage, read_write> mesh_vertex_locations: array<vec4f>;

@group(0) @binding(3)
var<storage, read_write> mesh_vertex_normals: array<vec4f>;

@group(0) @binding(4)
var<storage, read_write> mesh_vertex_materials: array<u32>;

@group(0) @binding(5)
var<storage, read_write> mesh_vertex_map: array<u32>;

@group(0) @binding(6)
var<storage, read_write> mesh_vertex_num: atomic<u32>;

fn estimate_edge_cross_point(
    voxel_cross_point_data: ptr<function, array<VoxelEdgeCrossPoint, 12>>,
    voxel_vertex_locations: ptr<function, array<vec3f, 8>>,
    voxel_vertex_values: ptr<function, array<f32, 8>>,
    left_vertex_index: u32,
    right_vertex_index: u32,
    edge_index: u32,
    voxel_size: f32,
) {
    let s1 = (*voxel_vertex_values)[left_vertex_index];
    let s2 = (*voxel_vertex_values)[right_vertex_index];
    let location_1 = (*voxel_vertex_locations)[left_vertex_index];
    let location_2 = (*voxel_vertex_locations)[right_vertex_index];
    if (s1 < 0.0 && s2 >= 0.0) || (s1 >= 0.0 && s2 < 0.0) {

        var dir = select(-1.0, 1.0, s2 > s1);

        var cross_pos = location_1 + (location_2 - location_1) * 0.5;
        var step = (location_2 - location_1) / 4.0;
        var cross_value = get_terrain_noise(cross_pos);
        for (var j = 0u; j < 8u; j++) {
            var offset_dir = select(-dir, dir, cross_value < 0.0);
            cross_pos += offset_dir * step;
            cross_value = get_terrain_noise(cross_pos);
            step /= 2.0;
        }

        // 因为有一个必为Air，不需要记录
        let s1_material_type_index = get_voxel_material_type_index(s1);
        let s2_material_type_index = get_voxel_material_type_index(s2);
        let material_index = max(s1_material_type_index, s2_material_type_index);

        let normal = central_gradient(cross_pos, voxel_size);
        (*voxel_cross_point_data)[edge_index] = VoxelEdgeCrossPoint(vec4f(cross_pos, 1.0), vec4f(normal, f32(material_index)));
    } else {
        (*voxel_cross_point_data)[edge_index] = VoxelEdgeCrossPoint(vec4f(0.0, 0.0, 0.0, 0.0), vec4f(0.0, 0.0, 0.0, f32(VOXEL_MATERIAL_AIR_INDEX)));
    }
}

fn compute_voxel_cross_points(
    voxel_cross_point_data: ptr<function, array<VoxelEdgeCrossPoint, 12>>,
    voxel_vertex_locations: ptr<function, array<vec3f, 8>>,
    voxel_vertex_values: ptr<function, array<f32, 8>>,
    voxel_size: f32
) {
    for (var i = 0u; i < 12u; i++) {
        let vertices_pairs = EDGE_VERTEX_PAIRS[i];
        estimate_edge_cross_point(voxel_cross_point_data, voxel_vertex_locations, voxel_vertex_values, vertices_pairs.x, vertices_pairs.y, i, voxel_size);
    }
}

fn compute_voxel_vertices(
    voxel_vertex_locations: ptr<function, array<vec3f, 8>>,
    voxel_vertex_values: ptr<function, array<f32, 8>>,
    min_location: vec3f,
    voxel_size: f32
) {
    for (var i = 0u; i < 8u; i++) {
        let vertex_location = min_location + VOXEL_VERTEX_OFFSETS[i] * voxel_size;
        (*voxel_vertex_locations)[i] = vertex_location;
        (*voxel_vertex_values)[i] = get_terrain_noise(vertex_location);
    }
}

fn compute_cross_point_data(
    voxel_cross_point_data: ptr<function, array<VoxelEdgeCrossPoint, 12>>,
    edge_index: u32,
    qef: ptr<function, Quadric>,
    location: ptr<function, vec4f>,
    normal: ptr<function, vec4f>,
    materials_count: ptr<function, array<vec2u, VOXEL_MATERIAL_NUM>>
) {
    let cross_point = (*voxel_cross_point_data)[edge_index];

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

fn compute_voxel_internal_vertices(
    voxel_cross_point_data: ptr<function, array<VoxelEdgeCrossPoint, 12>>,
    invocation_id: vec3u,
    coord_stride: vec3u,
    seam_chunk_size: vec3u,
) {
    var qef = quadric_default();
    var avg_location = vec4<f32>();
    var avg_normal = vec4<f32>();
    var materials_count = array<vec2u, VOXEL_MATERIAL_NUM>();
    for (var i = 0u; i < 12u; i++) {
        compute_cross_point_data(voxel_cross_point_data, i, &qef, &avg_location, &avg_normal, &materials_count);
    }

    let index = coord_stride.x * coord_stride.y * coord_stride.z;
    let stride = vec2u(coord_stride.y * coord_stride.z, coord_stride.z);
    let seam_voxel_total_num = seam_chunk_size.x * seam_chunk_size.y * seam_chunk_size.z;

    let count = avg_location.w;
    if count <= 0.0 {
        for (var i = 0u; i < index; i++) {
            let x = i / stride[0];
            let y = (i - x * stride[0]) / stride[1];
            let z = i - x * stride[0] - y * stride[1];
            if invocation_id.x + x < seam_chunk_size.x && invocation_id.y + y < seam_chunk_size.y && invocation_id.z + z < seam_chunk_size.z {
                let voxel_index = get_voxel_internal_vertex_index(
                    seam_chunk_size,
                    invocation_id.x + x,
                    invocation_id.y + y,
                    invocation_id.z + z,
                );

                if seam_voxel_total_num > voxel_index {
                    mesh_vertex_map[voxel_index] = U32_MAX;
                }
            }
        }
        return;
    }

    var qef_location = quadric_minimizer(qef);
    avg_location = select(
        avg_location / count,
        vec4f(qef_location, 1.0),
        quadric_residual_l2_error(qef, qef_location) < terrain_chunk_info.qef_threshold
    );

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

    let vertex_index = atomicAdd(&mesh_vertex_num, 1u);

    mesh_vertex_locations[vertex_index] = avg_location;
    mesh_vertex_normals[vertex_index] = avg_normal;
    mesh_vertex_materials[vertex_index] = material;
    for (var i = 0u; i < index; i++) {
        let x = i / stride[0];
        let y = (i - x * stride[0]) / stride[1];
        let z = i - x * stride[0] - y * stride[1];
        if invocation_id.x + x < seam_chunk_size.x && invocation_id.y + y < seam_chunk_size.y && invocation_id.z + z < seam_chunk_size.z {
            let voxel_index = get_voxel_internal_vertex_index(
                seam_chunk_size,
                invocation_id.x + x,
                invocation_id.y + y,
                invocation_id.z + z,
            );

            if seam_voxel_total_num > voxel_index {
                mesh_vertex_map[voxel_index] = vertex_index;
            }
        }
    }
}

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
@compute @workgroup_size(2, 1, 1)
fn compute_vertices_x_axis(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    // 包括了外部chunk的一个面的体素
    if (invocation_id.y > terrain_chunk_info.voxel_num) || (invocation_id.z > terrain_chunk_info.voxel_num) {
        return;
    }

    let seam_voxel_size = vec3u(2, terrain_chunk_info.voxel_num + 1, terrain_chunk_info.voxel_num + 1);

    // 缝隙chunk的最小位置
    let seam_base_offset = vec3u(1u, 0u, 0u) * (terrain_chunk_info.voxel_num - 1);
    // 整个chunk的坐标
    let seam_coord = seam_base_offset + invocation_id;

    // chunk外面的voxel，结果是1
    let x = seam_coord.x / terrain_chunk_info.voxel_num;
    let y = seam_coord.y / terrain_chunk_info.voxel_num;
    let z = seam_coord.z / terrain_chunk_info.voxel_num;
    let index = x + y * 2 + z * 4;

    let half_voxel_num = terrain_chunk_info.voxel_num / 2;
    let h_x = (seam_coord.x - x * terrain_chunk_info.voxel_num) / half_voxel_num;
    let h_y = (seam_coord.y - y * terrain_chunk_info.voxel_num) / half_voxel_num;
    let h_z = (seam_coord.z - z * terrain_chunk_info.voxel_num) / half_voxel_num;
    let lod_index = h_x + h_y * 2;
    let lod = terrain_chunks_lod[index * 2 + h_z][lod_index];

    let lod_scale = round(pow(2.0, f32(lod)));
    let coord_stride = vec3u(1u, u32(lod_scale), u32(lod_scale));

    if (invocation_id.y % coord_stride.y != 0) || (invocation_id.z % coord_stride.z != 0) {
        return;
    }

    let voxel_size = terrain_chunk_info.voxel_size * lod_scale;

    // 当前体素的最小位置
    let chunk_base_offset = vec3u(1u, 0u, 0u) * terrain_chunk_info.voxel_num;
    let chunk_coord = chunk_base_offset + vec3u(0u, invocation_id.y, invocation_id.z);
    let chunk_voxel_edge_location = terrain_chunk_info.chunk_min_location_size.xyz + vec3f(chunk_coord) * terrain_chunk_info.voxel_size;

    var seam_voxel_min_location = select(
        chunk_voxel_edge_location,
        chunk_voxel_edge_location - vec3f(voxel_size, 0.0, 0.0),
        invocation_id.x == 0
    );

    var voxel_vertex_locations = array<vec3f, 8>();
    var voxel_vertex_values = array<f32, 8>();
    var voxel_cross_point_data = array<VoxelEdgeCrossPoint, 12>();

    compute_voxel_vertices(&voxel_vertex_locations, &voxel_vertex_values, seam_voxel_min_location, voxel_size);
    compute_voxel_cross_points(&voxel_cross_point_data, &voxel_vertex_locations, &voxel_vertex_values, voxel_size);
    compute_voxel_internal_vertices(&voxel_cross_point_data, invocation_id, coord_stride, seam_voxel_size);
}

@compute @workgroup_size(1, 2, 1)
fn compute_vertices_y_axis(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    // 包括了外部chunk的一个面的体素
    if (invocation_id.x > terrain_chunk_info.voxel_num) || (invocation_id.z > terrain_chunk_info.voxel_num) {
        return;
    }

    let seam_voxel_size = vec3u(terrain_chunk_info.voxel_num + 1, 2, terrain_chunk_info.voxel_num + 1);

    // 缝隙chunk的最小位置
    let seam_base_offset = vec3u(0u, 1u, 0u) * (terrain_chunk_info.voxel_num - 1);
    // 整个chunk的坐标
    let seam_coord = seam_base_offset + invocation_id;

    // chunk外面的voxel，结果是1
    let x = seam_coord.x / terrain_chunk_info.voxel_num;
    let y = seam_coord.y / terrain_chunk_info.voxel_num;
    let z = seam_coord.z / terrain_chunk_info.voxel_num;
    let index = x + y * 2 + z * 4;

    let half_voxel_num = terrain_chunk_info.voxel_num / 2;
    let h_x = (seam_coord.x - x * terrain_chunk_info.voxel_num) / half_voxel_num;
    let h_y = (seam_coord.y - y * terrain_chunk_info.voxel_num) / half_voxel_num;
    let h_z = (seam_coord.z - z * terrain_chunk_info.voxel_num) / half_voxel_num;
    let lod_index = h_x + h_y * 2;
    let lod = terrain_chunks_lod[index * 2 + h_z][lod_index];

    let lod_scale = round(pow(2.0, f32(lod)));
    let coord_stride = vec3u(u32(lod_scale), 1u, u32(lod_scale));

    if (invocation_id.x % coord_stride.x != 0) || (invocation_id.z % coord_stride.z != 0) {
        return;
    }

    let voxel_size = terrain_chunk_info.voxel_size * lod_scale;

    // 当前体素的最小位置
    let chunk_base_offset = vec3u(0u, 1u, 0u) * terrain_chunk_info.voxel_num;
    let chunk_coord = chunk_base_offset + vec3u(invocation_id.x, 0u, invocation_id.z);
    let chunk_voxel_edge_location = terrain_chunk_info.chunk_min_location_size.xyz + vec3f(chunk_coord) * terrain_chunk_info.voxel_size;

    var seam_voxel_min_location = select(
        chunk_voxel_edge_location,
        chunk_voxel_edge_location - vec3f(0.0, voxel_size, 0.0),
        invocation_id.y == 0
    );

    var voxel_vertex_locations = array<vec3f, 8>();
    var voxel_vertex_values = array<f32, 8>();
    var voxel_cross_point_data = array<VoxelEdgeCrossPoint, 12>();

    compute_voxel_vertices(&voxel_vertex_locations, &voxel_vertex_values, seam_voxel_min_location, voxel_size);
    compute_voxel_cross_points(&voxel_cross_point_data, &voxel_vertex_locations, &voxel_vertex_values, voxel_size);
    compute_voxel_internal_vertices(&voxel_cross_point_data, invocation_id, coord_stride, seam_voxel_size);
}

@compute @workgroup_size(1, 1, 2)
fn compute_vertices_z_axis(@builtin(global_invocation_id) invocation_id: vec3<u32>) {
    // 包括了外部chunk的一个面的体素
    if (invocation_id.x > terrain_chunk_info.voxel_num) || (invocation_id.y > terrain_chunk_info.voxel_num) {
        return;
    }

    let seam_chunk_size = vec3u(terrain_chunk_info.voxel_num + 1, terrain_chunk_info.voxel_num + 1, 2);

    // 缝隙chunk的最小位置
    let seam_base_offset = vec3u(0u, 0u, 1u) * (terrain_chunk_info.voxel_num - 1);
    // 整个chunk的坐标
    let seam_coord = seam_base_offset + invocation_id;

    // chunk外面的voxel，结果是1
    let x = seam_coord.x / terrain_chunk_info.voxel_num;
    let y = seam_coord.y / terrain_chunk_info.voxel_num;
    let z = seam_coord.z / terrain_chunk_info.voxel_num;
    let index = x + y * 2 + z * 4;

    let half_voxel_num = terrain_chunk_info.voxel_num / 2;
    let h_x = (seam_coord.x - x * terrain_chunk_info.voxel_num) / half_voxel_num;
    let h_y = (seam_coord.y - y * terrain_chunk_info.voxel_num) / half_voxel_num;
    let h_z = (seam_coord.z - z * terrain_chunk_info.voxel_num) / half_voxel_num;
    let lod_index = h_x + h_y * 2;
    let lod = terrain_chunks_lod[index * 2 + h_z][lod_index];

    let lod_scale = round(pow(2.0, f32(lod)));
    let coord_stride = vec3u(u32(lod_scale), u32(lod_scale), 1u);

    if (invocation_id.x % coord_stride.x != 0) || (invocation_id.y % coord_stride.y != 0) {
        return;
    }

    let voxel_size = terrain_chunk_info.voxel_size * lod_scale;

    // 当前体素的最小位置
    let chunk_base_offset = vec3u(0u, 0u, 1u) * terrain_chunk_info.voxel_num;
    let chunk_coord = chunk_base_offset + vec3u(invocation_id.x, invocation_id.y, 0u);
    let chunk_voxel_edge_location = terrain_chunk_info.chunk_min_location_size.xyz + vec3f(chunk_coord) * terrain_chunk_info.voxel_size;

    var seam_voxel_min_location = select(
        chunk_voxel_edge_location,
        chunk_voxel_edge_location - vec3f(0.0, 0.0, voxel_size),
        invocation_id.z == 0
    );

    var voxel_vertex_locations = array<vec3f, 8>();
    var voxel_vertex_values = array<f32, 8>();
    var voxel_cross_point_data = array<VoxelEdgeCrossPoint, 12>();

    compute_voxel_vertices(&voxel_vertex_locations, &voxel_vertex_values, seam_voxel_min_location, voxel_size);
    compute_voxel_cross_points(&voxel_cross_point_data, &voxel_vertex_locations, &voxel_vertex_values, voxel_size);
    compute_voxel_internal_vertices(&voxel_cross_point_data, invocation_id, coord_stride, seam_chunk_size);
}
