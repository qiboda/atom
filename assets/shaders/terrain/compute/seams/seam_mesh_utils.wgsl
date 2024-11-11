#define_import_path terrain::seam_utils
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
// x y z axis
var<private> axis_base_offset: array<vec3u, 3> = array<vec3u, 3>(
    vec3u(1, 0, 0),
    vec3u(0, 1, 0),
    vec3u(0, 0, 1),
);

var<private> coord_convert: array<array<vec3u, 3>, 3> = array<array<vec3u, 3>, 3>(
    array(vec3u(0, 1, 0), vec3u(0, 0, 1), vec3u(1, 0, 0)),
    array(vec3u(1, 0, 0), vec3u(0, 0, 1), vec3u(0, 1, 0)),
    array(vec3u(1, 0, 0), vec3u(0, 1, 0), vec3u(0, 0, 1)),
);

fn coord_convert_fns(axis: u32, x: u32, y: u32, z: u32) -> vec3u {
    return coord_convert[axis][0] * x + coord_convert[axis][1] * y + coord_convert[axis][0] * z;
}


// voxel_num is chunk voxel num and is not lod voxel num
fn get_voxel_internal_vertex_index(
    voxel_num: vec3u,
    x: u32, y: u32, z: u32
) -> u32 {
    return x + y * voxel_num.x + z * voxel_num.x * voxel_num.y;
}

var<private> VOXEL_VERTEX_OFFSETS: array<vec3<f32>, 8u> = array<vec3<f32>, 8u>(
    vec3<f32>(0.0, 0.0, 0.0),
    vec3<f32>(1.0, 0.0, 0.0),
    vec3<f32>(0.0, 1.0, 0.0),
    vec3<f32>(1.0, 1.0, 0.0),
    vec3<f32>(0.0, 0.0, 1.0),
    vec3<f32>(1.0, 0.0, 1.0),
    vec3<f32>(0.0, 1.0, 1.0),
    vec3<f32>(1.0, 1.0, 1.0),
);

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
// x y z axis
var<private> EDGE_VERTEX_PAIRS: array<vec2<u32>, 12u> = array<vec2<u32>, 12u>(
    // x axis
    vec2<u32>(0u, 1u),
    vec2<u32>(2u, 3u),
    vec2<u32>(4u, 5u),
    vec2<u32>(6u, 7u),
    // y axis
    vec2<u32>(0u, 2u),
    vec2<u32>(1u, 3u),
    vec2<u32>(4u, 6u),
    vec2<u32>(5u, 7u),
    // z axis
    vec2<u32>(0u, 4u),
    vec2<u32>(1u, 5u),
    vec2<u32>(2u, 6u),
    vec2<u32>(3u, 7u),
);