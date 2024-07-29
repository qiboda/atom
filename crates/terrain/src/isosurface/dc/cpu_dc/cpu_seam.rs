use bevy::{
    math::{UVec2, UVec3, UVec4, Vec2, Vec3, Vec4, Vec4Swizzles},
    prelude::Mesh,
    render::{mesh::Indices, render_asset::RenderAssetUsages},
};
use pqef::quadric::Quadric;
use tracing::{error, info};
use wgpu::PrimitiveTopology;

use crate::{
    isosurface::{
        dc::gpu_dc::buffer_cache::{TerrainChunkInfo, VoxelEdgeCrossPoint},
        materials::terrain_mat::MATERIAL_VERTEX_ATTRIBUTE,
    },
    tables::EDGE_NODES_VERTICES,
};

const U32_MAX: u32 = 0xFFFFFFFF;
const VOXEL_MATERIAL_AIR: u32 = 0x0;
const VOXEL_MATERIAL_BLOCK: u32 = 0x1;

const VOXEL_MATERIAL_NUM: usize = 2;
// 不包括air
const VOXEL_MATERIAL_BLOCK_NUM: usize = 1;

const VOXEL_MATERIAL_AIR_INDEX: usize = 0;

const VOXEL_MATERIAL_TABLE: [u32; VOXEL_MATERIAL_NUM] = [VOXEL_MATERIAL_AIR, VOXEL_MATERIAL_BLOCK];

fn get_voxel_material_type(value: f32) -> u32 {
    if value >= 0.0 {
        return VOXEL_MATERIAL_AIR;
    }
    return VOXEL_MATERIAL_BLOCK;
}

fn get_voxel_material_type_index(value: f32) -> u32 {
    if value >= 0.0 {
        return 0;
    }
    return 1;
}

// size: terrain_chunk_info.voxel_size;
fn central_gradient(p: Vec3, size: f32) -> Vec3 {
    let h = 0.5 * size;
    let x = get_terrain_noise(p + Vec3::new(h, 0.0, 0.0))
        - get_terrain_noise(p - Vec3::new(h, 0.0, 0.0));
    let y = get_terrain_noise(p + Vec3::new(0.0, h, 0.0))
        - get_terrain_noise(p - Vec3::new(0.0, h, 0.0));
    let z = get_terrain_noise(p + Vec3::new(0.0, 0.0, h))
        - get_terrain_noise(p - Vec3::new(0.0, 0.0, h));
    return Vec3::new(x, y, z).normalize();
}

// voxel_num is chunk voxel num and is not lod voxel num
fn get_voxel_internal_vertex_index(voxel_num: UVec3, x: u32, y: u32, z: u32) -> usize {
    return (x + y * voxel_num.x + z * voxel_num.x * voxel_num.y) as usize;
}

fn plane(location: Vec3, normal: Vec3, height: f32) -> f32 {
    // n must be normalized
    return location.dot(normal) + height;
}

fn cube(position: Vec3, half_size: Vec3) -> f32 {
    let q = position.abs() - half_size;
    return q.max(Vec3::new(0.0, 0.0, 0.0)).length() + q.x.max(q.y).max(q.z).min(0.0);
}

fn get_terrain_noise(location: Vec3) -> f32 {
    return plane(location, Vec3::new(0.0, 1.0, 0.0), 2.0);
    // let loc = location + Vec3::new(6.0, 6.0, 6.0);
    // return cube(loc, Vec3::new(14.0, 14.0, 14.0));
}

const VOXEL_VERTEX_OFFSETS: [Vec3; 8] = [
    Vec3::new(0.0, 0.0, 0.0),
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(1.0, 1.0, 0.0),
    Vec3::new(0.0, 0.0, 1.0),
    Vec3::new(1.0, 0.0, 1.0),
    Vec3::new(0.0, 1.0, 1.0),
    Vec3::new(1.0, 1.0, 1.0),
];

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
const EDGE_VERTEX_PAIRS: [UVec2; 12] = [
    // x axis
    UVec2::new(0, 1),
    UVec2::new(2, 3),
    UVec2::new(4, 5),
    UVec2::new(6, 7),
    // y axis
    UVec2::new(0, 2),
    UVec2::new(1, 3),
    UVec2::new(4, 6),
    UVec2::new(5, 7),
    // z axis
    UVec2::new(0, 4),
    UVec2::new(1, 5),
    UVec2::new(2, 6),
    UVec2::new(3, 7),
];

pub struct ComputeIndicesContext {
    pub mesh_vertex_locations: Vec<Vec3>,
    pub mesh_vertex_map: Vec<usize>,
    pub mesh_indices_data: Vec<usize>,
    pub mesh_indices_num: usize,
}

impl ComputeIndicesContext {
    pub fn log(&self, terrain_chunk_info: &TerrainChunkInfo, axis: usize) {
        let voxel_num = terrain_chunk_info.voxel_num;
        let seam_voxel_num = if axis == 0 {
            UVec3::new(2, voxel_num + 1, voxel_num + 1)
        } else if axis == 1 {
            UVec3::new(voxel_num + 1, 2, voxel_num + 1)
        } else {
            UVec3::new(voxel_num + 1, voxel_num + 1, 2)
        };

        if axis == 0 {
            for x in 0..seam_voxel_num.x {
                for y in 0..seam_voxel_num.y {
                    for z in 0..seam_voxel_num.z {
                        let index = get_voxel_internal_vertex_index(seam_voxel_num, x, y, z);
                        info!(
                            "chunk location: {}, axis: {}, index: {}:[{},{},{}], vertex map: {}",
                            terrain_chunk_info.chunk_min_location_size,
                            axis,
                            index,
                            x,
                            y,
                            z,
                            self.mesh_vertex_map[index]
                        );
                    }
                }
            }
        } else if axis == 1 {
            for y in 0..seam_voxel_num.y {
                for x in 0..seam_voxel_num.x {
                    for z in 0..seam_voxel_num.z {
                        let index = get_voxel_internal_vertex_index(seam_voxel_num, x, y, z);
                        info!(
                            "chunk location: {}, axis: {}, index: {}:[{},{},{}], vertex map: {}",
                            terrain_chunk_info.chunk_min_location_size,
                            axis,
                            index,
                            x,
                            y,
                            z,
                            self.mesh_vertex_map[index]
                        );
                    }
                }
            }
        } else if axis == 2 {
            for z in 0..seam_voxel_num.z {
                for x in 0..seam_voxel_num.x {
                    for y in 0..seam_voxel_num.y {
                        let index = get_voxel_internal_vertex_index(seam_voxel_num, x, y, z);
                        info!(
                            "chunk location: {}, axis: {}, index: {}:[{},{},{}], vertex map: {}",
                            terrain_chunk_info.chunk_min_location_size,
                            axis,
                            index,
                            x,
                            y,
                            z,
                            self.mesh_vertex_map[index]
                        );
                    }
                }
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

// chunk的z axis方向的缝隙voxel。
fn compute_indices_z_axis(
    context: &mut ComputeIndicesContext,
    terrain_chunk_info: &TerrainChunkInfo,
    invocation_id: UVec3,
) {
    if (invocation_id.x > terrain_chunk_info.voxel_num)
        || (invocation_id.y > terrain_chunk_info.voxel_num)
    {
        return;
    }

    // use select(true, false, condition) to get 1 or 0
    let mut xyz = UVec3::ZERO;
    if invocation_id.x == terrain_chunk_info.voxel_num {
        xyz.x = 1;
    }
    if invocation_id.y == terrain_chunk_info.voxel_num {
        xyz.y = 1;
    }
    if invocation_id.z == 1 {
        xyz.z = 1;
    }

    let seam_voxel_size = UVec3::new(
        terrain_chunk_info.voxel_num + 1,
        terrain_chunk_info.voxel_num + 1,
        2,
    );

    // 缝隙chunk的最小位置
    let chunk_base_offset = UVec3::new(0, 0, 1) * (terrain_chunk_info.voxel_num - 1);
    // 整个chunk的坐标
    let chunk_coord = chunk_base_offset + invocation_id;
    // 当前体素的最小位置
    let seam_voxel_min_location = terrain_chunk_info.chunk_min_location_size.xyz()
        + Vec3::new(
            chunk_coord.x as f32,
            chunk_coord.y as f32,
            chunk_coord.z as f32,
        ) * terrain_chunk_info.voxel_size;

    let vertex_index_0 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x,
        invocation_id.y,
        invocation_id.z,
    );
    let invocation_id_0 = invocation_id;
    let vertex_index_1 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x + 1,
        invocation_id.y,
        invocation_id.z,
    );
    let invocation_id_1 = invocation_id + UVec3::new(1, 0, 0);
    let vertex_index_2 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x,
        invocation_id.y + 1,
        invocation_id.z,
    );
    let invocation_id_2 = invocation_id + UVec3::new(0, 1, 0);
    let vertex_index_3 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x + 1,
        invocation_id.y + 1,
        invocation_id.z,
    );
    let invocation_id_3 = invocation_id + UVec3::new(1, 1, 0);
    let vertex_index_4 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x,
        invocation_id.y,
        invocation_id.z + 1,
    );
    let invocation_id_4 = invocation_id + UVec3::new(0, 0, 1);
    let vertex_index_5 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x + 1,
        invocation_id.y,
        invocation_id.z + 1,
    );
    let invocation_id_5 = invocation_id + UVec3::new(1, 0, 1);
    let vertex_index_6 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x,
        invocation_id.y + 1,
        invocation_id.z + 1,
    );
    let invocation_id_6 = invocation_id + UVec3::new(0, 1, 1);

    let voxel_vertex_value_3 = get_terrain_noise(
        seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[3],
    );
    let voxel_vertex_value_5 = get_terrain_noise(
        seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[5],
    );
    let voxel_vertex_value_6 = get_terrain_noise(
        seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[6],
    );
    let voxel_vertex_value_7 = get_terrain_noise(
        seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[7],
    );

    // x axis 0 2 4 6
    if xyz.y == 0 && xyz.z == 0 {
        compute_indices_on_axis(
            context,
            terrain_chunk_info,
            voxel_vertex_value_7,
            voxel_vertex_value_6,
            vertex_index_0,
            vertex_index_2,
            vertex_index_4,
            vertex_index_6,
            invocation_id_0,
            invocation_id_2,
            invocation_id_4,
            invocation_id_6,
            terrain_chunk_info.voxel_num,
            2,
        );
    }
    // y axis 0 1 4 5
    if xyz.x == 0 && xyz.z == 0 {
        compute_indices_on_axis(
            context,
            terrain_chunk_info,
            voxel_vertex_value_5,
            voxel_vertex_value_7,
            vertex_index_0,
            vertex_index_1,
            vertex_index_4,
            vertex_index_5,
            invocation_id_0,
            invocation_id_1,
            invocation_id_4,
            invocation_id_5,
            terrain_chunk_info.voxel_num,
            2,
        );
    }
    // z axis 0 1 2 3
    if xyz.x == 0 && xyz.y == 0 {
        compute_indices_on_axis(
            context,
            terrain_chunk_info,
            voxel_vertex_value_7,
            voxel_vertex_value_3,
            vertex_index_0,
            vertex_index_1,
            vertex_index_2,
            vertex_index_3,
            invocation_id_0,
            invocation_id_1,
            invocation_id_2,
            invocation_id_3,
            terrain_chunk_info.voxel_num,
            2,
        );
    }
}

// chunk的x axis方向的缝隙voxel。
fn compute_indices_y_axis(
    context: &mut ComputeIndicesContext,
    terrain_chunk_info: &TerrainChunkInfo,
    invocation_id: UVec3,
) {
    if (invocation_id.x > terrain_chunk_info.voxel_num)
        || (invocation_id.z > terrain_chunk_info.voxel_num)
    {
        return;
    }

    let seam_voxel_size = UVec3::new(
        terrain_chunk_info.voxel_num + 1,
        2,
        terrain_chunk_info.voxel_num + 1,
    );

    let mut xyz = UVec3::new(0, 0, 0);
    if invocation_id.x == terrain_chunk_info.voxel_num {
        xyz.x = 1;
    }
    if invocation_id.y == 1 {
        xyz.y = 1;
    }
    if invocation_id.z == terrain_chunk_info.voxel_num {
        xyz.z = 1;
    }

    // 缝隙chunk的最小位置
    let chunk_base_offset = UVec3::new(0, 1, 0) * (terrain_chunk_info.voxel_num - 1);
    // 整个chunk的坐标
    let chunk_coord = chunk_base_offset + invocation_id;
    // 当前体素的最小位置
    let seam_voxel_min_location = terrain_chunk_info.chunk_min_location_size.xyz()
        + Vec3::new(
            chunk_coord.x as f32,
            chunk_coord.y as f32,
            chunk_coord.z as f32,
        ) * terrain_chunk_info.voxel_size;

    let vertex_index_0 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x,
        invocation_id.y,
        invocation_id.z,
    );
    let invocation_id_0 = invocation_id + UVec3::new(0, 0, 0);
    let vertex_index_1 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x + 1,
        invocation_id.y,
        invocation_id.z,
    );
    let invocation_id_1 = invocation_id + UVec3::new(1, 0, 0);
    let vertex_index_2 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x,
        invocation_id.y + 1,
        invocation_id.z,
    );
    let invocation_id_2 = invocation_id + UVec3::new(0, 1, 0);
    let vertex_index_3 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x + 1,
        invocation_id.y + 1,
        invocation_id.z,
    );
    let invocation_id_3 = invocation_id + UVec3::new(1, 1, 0);
    let vertex_index_4 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x,
        invocation_id.y,
        invocation_id.z + 1,
    );
    let invocation_id_4 = invocation_id + UVec3::new(0, 0, 1);
    let vertex_index_5 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x + 1,
        invocation_id.y,
        invocation_id.z + 1,
    );
    let invocation_id_5 = invocation_id + UVec3::new(1, 0, 1);
    let vertex_index_6 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x,
        invocation_id.y + 1,
        invocation_id.z + 1,
    );
    let invocation_id_6 = invocation_id + UVec3::new(0, 1, 1);

    let voxel_vertex_value_3 = get_terrain_noise(
        seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[3],
    );
    let voxel_vertex_value_5 = get_terrain_noise(
        seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[5],
    );
    let voxel_vertex_value_6 = get_terrain_noise(
        seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[6],
    );
    let voxel_vertex_value_7 = get_terrain_noise(
        seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[7],
    );

    // x axis 0 2 4 6
    if xyz.y == 0 && xyz.z == 0 {
        compute_indices_on_axis(
            context,
            terrain_chunk_info,
            voxel_vertex_value_7,
            voxel_vertex_value_6,
            vertex_index_0,
            vertex_index_2,
            vertex_index_4,
            vertex_index_6,
            invocation_id_0,
            invocation_id_2,
            invocation_id_4,
            invocation_id_6,
            terrain_chunk_info.voxel_num,
            1,
        );
    }
    // y axis 0 1 4 5
    if xyz.x == 0 && xyz.z == 0 {
        compute_indices_on_axis(
            context,
            terrain_chunk_info,
            voxel_vertex_value_5,
            voxel_vertex_value_7,
            vertex_index_0,
            vertex_index_1,
            vertex_index_4,
            vertex_index_5,
            invocation_id_0,
            invocation_id_1,
            invocation_id_4,
            invocation_id_5,
            terrain_chunk_info.voxel_num,
            1,
        );
    }
    // z axis 0 1 2 3
    if xyz.x == 0 && xyz.y == 0 {
        compute_indices_on_axis(
            context,
            terrain_chunk_info,
            voxel_vertex_value_7,
            voxel_vertex_value_3,
            vertex_index_0,
            vertex_index_1,
            vertex_index_2,
            vertex_index_3,
            invocation_id_0,
            invocation_id_1,
            invocation_id_2,
            invocation_id_3,
            terrain_chunk_info.voxel_num,
            1,
        );
    }
}

// chunk的x axis方向的缝隙voxel。
fn compute_indices_x_axis(
    context: &mut ComputeIndicesContext,
    terrain_chunk_info: &TerrainChunkInfo,
    invocation_id: UVec3,
) {
    if (invocation_id.y > terrain_chunk_info.voxel_num)
        || (invocation_id.z > terrain_chunk_info.voxel_num)
    {
        return;
    }

    let mut xyz = UVec3::new(0, 0, 0);
    if invocation_id.x == 1 {
        xyz.x = 1;
    }
    if invocation_id.y == terrain_chunk_info.voxel_num {
        xyz.y = 1;
    }
    if invocation_id.z == terrain_chunk_info.voxel_num {
        xyz.z = 1;
    }

    let seam_voxel_size = UVec3::new(
        2,
        terrain_chunk_info.voxel_num + 1,
        terrain_chunk_info.voxel_num + 1,
    );

    // 缝隙chunk的最小位置
    let chunk_base_offset = UVec3::new(1, 0, 0) * (terrain_chunk_info.voxel_num - 1);
    // 整个chunk的坐标
    let chunk_coord = chunk_base_offset + invocation_id;
    // 当前体素的最小位置
    let seam_voxel_min_location = terrain_chunk_info.chunk_min_location_size.xyz()
        + Vec3::new(
            chunk_coord.x as f32,
            chunk_coord.y as f32,
            chunk_coord.z as f32,
        ) * terrain_chunk_info.voxel_size;

    let vertex_index_0 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x,
        invocation_id.y,
        invocation_id.z,
    );
    let invocation_id_0 = invocation_id + UVec3::new(0, 0, 0);
    let vertex_index_1 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x + 1,
        invocation_id.y,
        invocation_id.z,
    );
    let invocation_id_1 = invocation_id + UVec3::new(1, 0, 0);
    let vertex_index_2 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x,
        invocation_id.y + 1,
        invocation_id.z,
    );
    let invocation_id_2 = invocation_id + UVec3::new(0, 1, 0);
    let vertex_index_3 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x + 1,
        invocation_id.y + 1,
        invocation_id.z,
    );
    let invocation_id_3 = invocation_id + UVec3::new(1, 1, 0);
    let vertex_index_4 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x,
        invocation_id.y,
        invocation_id.z + 1,
    );
    let invocation_id_4 = invocation_id + UVec3::new(0, 0, 1);
    let vertex_index_5 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x + 1,
        invocation_id.y,
        invocation_id.z + 1,
    );
    let invocation_id_5 = invocation_id + UVec3::new(1, 0, 1);
    let vertex_index_6 = get_voxel_internal_vertex_index(
        seam_voxel_size,
        invocation_id.x,
        invocation_id.y + 1,
        invocation_id.z + 1,
    );
    let invocation_id_6 = invocation_id + UVec3::new(0, 1, 1);

    let voxel_vertex_value_3 = get_terrain_noise(
        seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[3],
    );
    let voxel_vertex_value_5 = get_terrain_noise(
        seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[5],
    );
    let voxel_vertex_value_6 = get_terrain_noise(
        seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[6],
    );
    let voxel_vertex_value_7 = get_terrain_noise(
        seam_voxel_min_location + terrain_chunk_info.voxel_size * VOXEL_VERTEX_OFFSETS[7],
    );

    // x axis 0 2 4 6
    if xyz.y == 0 && xyz.z == 0 {
        compute_indices_on_axis(
            context,
            terrain_chunk_info,
            voxel_vertex_value_7,
            voxel_vertex_value_6,
            vertex_index_0,
            vertex_index_2,
            vertex_index_4,
            vertex_index_6,
            invocation_id_0,
            invocation_id_2,
            invocation_id_4,
            invocation_id_6,
            terrain_chunk_info.voxel_num,
            0,
        );
    }
    // y axis 0 1 4 5
    if xyz.x == 0 && xyz.z == 0 {
        compute_indices_on_axis(
            context,
            terrain_chunk_info,
            voxel_vertex_value_5,
            voxel_vertex_value_7,
            vertex_index_0,
            vertex_index_1,
            vertex_index_4,
            vertex_index_5,
            invocation_id_0,
            invocation_id_1,
            invocation_id_4,
            invocation_id_5,
            terrain_chunk_info.voxel_num,
            0,
        );
    }
    // z axis 0 1 2 3
    if xyz.x == 0 && xyz.y == 0 {
        compute_indices_on_axis(
            context,
            terrain_chunk_info,
            voxel_vertex_value_7,
            voxel_vertex_value_3,
            vertex_index_0,
            vertex_index_1,
            vertex_index_2,
            vertex_index_3,
            invocation_id_0,
            invocation_id_1,
            invocation_id_2,
            invocation_id_3,
            terrain_chunk_info.voxel_num,
            0,
        );
    }
}

fn set_value(index_array: &mut UVec4, value: usize) {
    let mut same_value = false;
    for i in 0..4 {
        same_value |= index_array[i] as usize == value;
    }
    if same_value {
        return;
    }

    for i in 0..4 {
        if index_array[i] == U32_MAX && value as u32 != U32_MAX {
            index_array[i] = value as u32;
            break;
        }
    }
}

fn compute_indices_on_axis(
    context: &mut ComputeIndicesContext,
    terrain_chunk_info: &TerrainChunkInfo,
    vertex_value_0: f32,
    vertex_value_1: f32,
    voxel_index_0: usize,
    voxel_index_1: usize,
    voxel_index_2: usize,
    voxel_index_3: usize,
    invocation_0: UVec3,
    invocation_1: UVec3,
    invocation_2: UVec3,
    invocation_3: UVec3,
    voxel_num: u32,
    axis: usize,
) {
    if (vertex_value_0 >= 0.0 && vertex_value_1 >= 0.0)
        || (vertex_value_0 < 0.0 && vertex_value_1 < 0.0)
    {
        return;
    }

    let mesh_vertex_index_0 = context.mesh_vertex_map[voxel_index_0];
    let mesh_vertex_index_1 = context.mesh_vertex_map[voxel_index_1];
    let mesh_vertex_index_2 = context.mesh_vertex_map[voxel_index_2];
    let mesh_vertex_index_3 = context.mesh_vertex_map[voxel_index_3];

    let mut mesh_vertex_index_array = UVec4::new(U32_MAX, U32_MAX, U32_MAX, U32_MAX);
    set_value(&mut mesh_vertex_index_array, mesh_vertex_index_0);
    set_value(&mut mesh_vertex_index_array, mesh_vertex_index_1);
    set_value(&mut mesh_vertex_index_array, mesh_vertex_index_3);
    set_value(&mut mesh_vertex_index_array, mesh_vertex_index_2);

    let mut num = 0;
    for i in 0..4 {
        if mesh_vertex_index_array[i] != U32_MAX {
            num += 1;
        }
    }

    // info!("num == {}, {}", num, mesh_vertex_index_array);

    if num == 3 {
        let mesh_indices_index = context.mesh_indices_num;
        context.mesh_indices_num += 3;

        if vertex_value_0 >= 0.0 {
            context.mesh_indices_data[mesh_indices_index] = mesh_vertex_index_array[0] as usize;
            context.mesh_indices_data[mesh_indices_index + 1] = mesh_vertex_index_array[1] as usize;
            context.mesh_indices_data[mesh_indices_index + 2] = mesh_vertex_index_array[2] as usize;
        } else {
            context.mesh_indices_data[mesh_indices_index] = mesh_vertex_index_array[0] as usize;
            context.mesh_indices_data[mesh_indices_index + 1] = mesh_vertex_index_array[2] as usize;
            context.mesh_indices_data[mesh_indices_index + 2] = mesh_vertex_index_array[1] as usize;
        }
    } else if num == 4 {
        let mesh_indices_index = context.mesh_indices_num;
        context.mesh_indices_num += 6;

        if vertex_value_0 >= 0.0 {
            context.mesh_indices_data[mesh_indices_index] = mesh_vertex_index_array[0] as usize;
            context.mesh_indices_data[mesh_indices_index + 1] = mesh_vertex_index_array[1] as usize;
            context.mesh_indices_data[mesh_indices_index + 2] = mesh_vertex_index_array[2] as usize;

            context.mesh_indices_data[mesh_indices_index + 3] = mesh_vertex_index_array[0] as usize;
            context.mesh_indices_data[mesh_indices_index + 4] = mesh_vertex_index_array[2] as usize;
            context.mesh_indices_data[mesh_indices_index + 5] = mesh_vertex_index_array[3] as usize;
        } else {
            context.mesh_indices_data[mesh_indices_index] = mesh_vertex_index_array[0] as usize;
            context.mesh_indices_data[mesh_indices_index + 1] = mesh_vertex_index_array[2] as usize;
            context.mesh_indices_data[mesh_indices_index + 2] = mesh_vertex_index_array[1] as usize;

            context.mesh_indices_data[mesh_indices_index + 3] = mesh_vertex_index_array[0] as usize;
            context.mesh_indices_data[mesh_indices_index + 4] = mesh_vertex_index_array[3] as usize;
            context.mesh_indices_data[mesh_indices_index + 5] = mesh_vertex_index_array[2] as usize;
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComputeVertexContext {
    pub mesh_vertex_locations: Vec<Vec4>,
    pub mesh_vertex_normals: Vec<Vec4>,
    pub mesh_vertex_materials: Vec<u32>,
    pub mesh_vertex_num: usize,
    pub mesh_vertex_map: Vec<usize>,
    pub terrain_chunks_lod: [UVec4; 16],
}

impl ComputeVertexContext {
    pub fn log(&self, terrain_chunk_info: &TerrainChunkInfo, axis: usize) {
        let voxel_num = terrain_chunk_info.voxel_num;
        let seam_voxel_num = if axis == 0 {
            UVec3::new(2, voxel_num + 1, voxel_num + 1)
        } else if axis == 1 {
            UVec3::new(voxel_num + 1, 2, voxel_num + 1)
        } else {
            UVec3::new(voxel_num + 1, voxel_num + 1, 2)
        };

        for x in 0..seam_voxel_num.x {
            for y in 0..seam_voxel_num.y {
                for z in 0..seam_voxel_num.z {
                    let index = get_voxel_internal_vertex_index(seam_voxel_num, x, y, z);
                    info!(
                        "axis: {}, index: [{},{},{}], vertex map: {}",
                        axis, x, y, z, self.mesh_vertex_map[index]
                    );
                }
            }
        }
    }
}

fn estimate_edge_cross_point(
    voxel_cross_point_data: &mut [VoxelEdgeCrossPoint; 12],
    voxel_vertex_locations: &[Vec3; 8],
    voxel_vertex_values: &[f32; 8],
    left_vertex_index: usize,
    right_vertex_index: usize,
    edge_index: usize,
    voxel_size: f32,
) {
    let s1 = (*voxel_vertex_values)[left_vertex_index];
    let s2 = (*voxel_vertex_values)[right_vertex_index];
    let location_1 = (*voxel_vertex_locations)[left_vertex_index];
    let location_2 = (*voxel_vertex_locations)[right_vertex_index];
    if (s1 < 0.0 && s2 >= 0.0) || (s1 >= 0.0 && s2 < 0.0) {
        let mut dir = 1.0;
        if s2 > s1 {
            dir = 1.0;
        } else {
            dir = -1.0;
        }

        let mut cross_pos = location_1 + (location_2 - location_1) * 0.5;
        let mut step = (location_2 - location_1) / 4.0;
        let mut cross_value = get_terrain_noise(cross_pos);
        for j in 0..8 {
            if cross_value == 0.0 {
                break;
            } else {
                let mut offset_dir = dir;
                if cross_value < 0.0 {
                    offset_dir = dir;
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
        let material_index = s1_material_type_index.max(s2_material_type_index);

        let normal = central_gradient(cross_pos, voxel_size);
        voxel_cross_point_data[edge_index] = VoxelEdgeCrossPoint {
            cross_pos: Vec4::new(cross_pos.x, cross_pos.y, cross_pos.z, 1.0),
            normal_material_index: Vec4::new(normal.x, normal.y, normal.z, material_index as f32),
        };
    } else {
        voxel_cross_point_data[edge_index] = VoxelEdgeCrossPoint {
            cross_pos: Vec4::new(0.0, 0.0, 0.0, 0.0),
            normal_material_index: Vec4::new(0.0, 0.0, 0.0, VOXEL_MATERIAL_AIR_INDEX as f32),
        }
    }
}

fn compute_voxel_cross_points(
    voxel_cross_point_data: &mut [VoxelEdgeCrossPoint; 12],
    voxel_vertex_locations: &[Vec3; 8],
    voxel_vertex_values: &[f32; 8],
    voxel_size: f32,
) {
    for i in 0..12 {
        let vertices_pairs = EDGE_VERTEX_PAIRS[i];
        estimate_edge_cross_point(
            voxel_cross_point_data,
            voxel_vertex_locations,
            voxel_vertex_values,
            vertices_pairs.x as usize,
            vertices_pairs.y as usize,
            i,
            voxel_size,
        );
    }
}

fn compute_voxel_vertices(
    voxel_vertex_locations: &mut [Vec3; 8],
    voxel_vertex_values: &mut [f32; 8],
    min_location: Vec3,
    voxel_size: f32,
) {
    for i in 0..8 {
        let vertex_location = min_location + VOXEL_VERTEX_OFFSETS[i] * voxel_size;
        voxel_vertex_locations[i] = vertex_location;
        voxel_vertex_values[i] = get_terrain_noise(vertex_location);
    }
}

fn compute_cross_point_data(
    terrain_chunk_info: &TerrainChunkInfo,
    voxel_cross_point_data: &[VoxelEdgeCrossPoint; 12],
    edge_index: usize,
    qef: &mut Quadric,
    location: &mut Vec4,
    normal: &mut Vec4,
    materials_count: &mut [UVec2; VOXEL_MATERIAL_NUM],
) {
    let cross_point = voxel_cross_point_data[edge_index];

    if cross_point.cross_pos.w == 0.0 {
        return;
    }

    *location += cross_point.cross_pos;

    *normal += cross_point.normal_material_index;

    let quadric = Quadric::probabilistic_plane_quadric(
        cross_point.cross_pos.xyz().into(),
        cross_point.normal_material_index.xyz().into(),
        terrain_chunk_info.qef_stddev * terrain_chunk_info.voxel_size,
        terrain_chunk_info.qef_stddev,
    );
    *qef = *qef + quadric;

    let material_index = cross_point.normal_material_index.w as usize;
    materials_count[material_index] = materials_count[material_index] + 1;
}

fn compute_voxel_internal_vertices(
    context: &mut ComputeVertexContext,
    terrain_chunk_info: &TerrainChunkInfo,
    voxel_cross_point_data: &[VoxelEdgeCrossPoint; 12],
    invocation_id: UVec3,
    coord_stride: UVec3,
    seam_chunk_size: UVec3,
    axis: u32,
) {
    let mut qef = Quadric::default();
    let mut avg_location = Vec4::ZERO;
    let mut avg_normal = Vec4::ZERO;
    let mut materials_count = [UVec2::ZERO; VOXEL_MATERIAL_NUM];
    for i in 0..12 {
        compute_cross_point_data(
            terrain_chunk_info,
            voxel_cross_point_data,
            i,
            &mut qef,
            &mut avg_location,
            &mut avg_normal,
            &mut materials_count,
        );
    }

    let count = avg_location.w;
    if count <= 0.0 {
        let index = coord_stride.x * coord_stride.y * coord_stride.z;
        let stride = UVec3::new(coord_stride.y * coord_stride.z, coord_stride.z, 1);
        for i in 0..index {
            let x = i / stride[0];
            let y = (i - x * stride[0]) / stride[1];
            let z = i - x * stride[0] - y * stride[1];
            if invocation_id.x + x < seam_chunk_size.x
                && invocation_id.y + y < seam_chunk_size.y
                && invocation_id.z + z < seam_chunk_size.z
            {
                let voxel_index = get_voxel_internal_vertex_index(
                    seam_chunk_size,
                    invocation_id.x + x,
                    invocation_id.y + y,
                    invocation_id.z + z,
                );

                if seam_chunk_size.x * seam_chunk_size.y * seam_chunk_size.z > voxel_index as u32 {
                    context.mesh_vertex_map[voxel_index] = U32_MAX as usize;
                }
            }
        }

        return;
    }

    let qef_location = qef.minimizer();
    if qef.residual_l2_error(qef_location) < terrain_chunk_info.qef_threshold {
        avg_location = Vec4::new(qef_location.x, qef_location.y, qef_location.z, 1.0);
    } else {
        avg_location = avg_location / count;
    }

    avg_normal.w = 0.0;
    avg_normal = avg_normal.normalize();

    let mut max_count = 0;
    let mut material = VOXEL_MATERIAL_AIR;
    for i in 0..VOXEL_MATERIAL_NUM {
        if materials_count[i].y > max_count {
            max_count = materials_count[i].y;
            material = materials_count[i].x;
        }
    }

    let vertex_index = context.mesh_vertex_num;
    context.mesh_vertex_num += 1;

    context.mesh_vertex_locations[vertex_index] = avg_location;
    context.mesh_vertex_normals[vertex_index] = avg_normal;
    context.mesh_vertex_materials[vertex_index] = material;

    let mut voxel_index_vec = vec![];

    let index = coord_stride.x * coord_stride.y * coord_stride.z;
    let stride = UVec3::new(coord_stride.y * coord_stride.z, coord_stride.z, 1);
    for i in 0..index {
        let x = i / stride[0];
        let y = (i - x * stride[0]) / stride[1];
        let z = i - x * stride[0] - y * stride[1];
        if invocation_id.x + x < seam_chunk_size.x
            && invocation_id.y + y < seam_chunk_size.y
            && invocation_id.z + z < seam_chunk_size.z
        {
            let voxel_index = get_voxel_internal_vertex_index(
                seam_chunk_size,
                invocation_id.x + x,
                invocation_id.y + y,
                invocation_id.z + z,
            );

            if seam_chunk_size.x * seam_chunk_size.y * seam_chunk_size.z > voxel_index as u32 {
                context.mesh_vertex_map[voxel_index] = vertex_index;
                voxel_index_vec.push(voxel_index);
            } else {
                error!(
                    "voxel_index too big, max index: {}, voxel index: {}",
                    seam_chunk_size.x * seam_chunk_size.y * seam_chunk_size.z,
                    voxel_index
                );
            }
        }
    }

    // if axis == 2 {
    //     if terrain_chunk_info.chunk_min_location_size == Vec4::new(-16.0, -16.0, 16.0, 16.0)
    //         && (vertex_index == 2 || vertex_index == 3 || vertex_index == 1 || vertex_index == 64)
    //     {
    //         info!(
    //             "axis: {}, vertex num: {}, invocation_id: {}, voxel_index_vec: {:?}, locations: {}, count:{}, qef location: {}, voxel cross point data: {:?}",
    //             axis,
    //             vertex_index,
    //             invocation_id,
    //             voxel_index_vec,
    //             avg_location,
    //             count,
    //             qef_location,
    //             voxel_cross_point_data,
    //         );
    //     }
    // }
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
fn compute_vertices_x_axis(
    context: &mut ComputeVertexContext,
    terrain_chunk_info: &TerrainChunkInfo,
    invocation_id: UVec3,
) {
    // 包括了外部chunk的一个面的体素
    if (invocation_id.y > terrain_chunk_info.voxel_num)
        || (invocation_id.z > terrain_chunk_info.voxel_num)
    {
        return;
    }

    let seam_voxel_size = UVec3::new(
        2,
        terrain_chunk_info.voxel_num + 1,
        terrain_chunk_info.voxel_num + 1,
    );

    // 缝隙chunk的最小位置
    let seam_base_offset = UVec3::new(1, 0, 0) * (terrain_chunk_info.voxel_num - 1);
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
    let lod = context.terrain_chunks_lod[(index * 2 + h_z) as usize][lod_index as usize];

    let lod_scale = 2.0f32.powf(lod as f32).round();
    let coord_stride = UVec3::new(1, lod_scale as u32, lod_scale as u32);

    if (invocation_id.y % coord_stride.y != 0) || (invocation_id.z % coord_stride.z != 0) {
        return;
    }

    let voxel_size = terrain_chunk_info.voxel_size * lod_scale;

    // 当前体素的最小位置
    let chunk_base_offset = UVec3::new(1, 0, 0) * terrain_chunk_info.voxel_num;
    let chunk_coord = chunk_base_offset + UVec3::new(0, invocation_id.y, invocation_id.z);
    let chunk_voxel_edge_location = terrain_chunk_info.chunk_min_location_size.xyz()
        + Vec3::new(
            chunk_coord.x as f32,
            chunk_coord.y as f32,
            chunk_coord.z as f32,
        ) * terrain_chunk_info.voxel_size;

    let mut seam_voxel_min_location = Vec3::new(0.0, 0.0, 0.0);
    if invocation_id.x == 0 {
        seam_voxel_min_location = chunk_voxel_edge_location - Vec3::new(voxel_size, 0.0, 0.0);
    } else {
        seam_voxel_min_location = chunk_voxel_edge_location;
    }

    let mut voxel_vertex_locations = [Vec3::ZERO; 8];
    let mut voxel_vertex_values = [0.0; 8];
    let mut voxel_cross_point_data = [VoxelEdgeCrossPoint::default(); 12];

    compute_voxel_vertices(
        &mut voxel_vertex_locations,
        &mut voxel_vertex_values,
        seam_voxel_min_location,
        voxel_size,
    );
    compute_voxel_cross_points(
        &mut voxel_cross_point_data,
        &voxel_vertex_locations,
        &voxel_vertex_values,
        voxel_size,
    );
    compute_voxel_internal_vertices(
        context,
        terrain_chunk_info,
        &voxel_cross_point_data,
        invocation_id,
        coord_stride,
        seam_voxel_size,
        0,
    );
}

fn compute_vertices_y_axis(
    context: &mut ComputeVertexContext,
    terrain_chunk_info: &TerrainChunkInfo,
    invocation_id: UVec3,
) {
    // 包括了外部chunk的一个面的体素
    if (invocation_id.x > terrain_chunk_info.voxel_num)
        || (invocation_id.z > terrain_chunk_info.voxel_num)
    {
        return;
    }

    let seam_voxel_size = UVec3::new(
        terrain_chunk_info.voxel_num + 1,
        2,
        terrain_chunk_info.voxel_num + 1,
    );

    // 缝隙chunk的最小位置
    let seam_base_offset = UVec3::new(0, 1, 0) * (terrain_chunk_info.voxel_num - 1);
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
    let lod = context.terrain_chunks_lod[(index * 2 + h_z) as usize][lod_index as usize];

    let lod_scale = 2.0f32.powf(lod as f32).round();
    let coord_stride = UVec3::new(lod_scale as u32, 1, lod_scale as u32);

    if (invocation_id.x % coord_stride.x != 0) || (invocation_id.z % coord_stride.z != 0) {
        return;
    }

    let voxel_size = terrain_chunk_info.voxel_size * lod_scale;

    // 当前体素的最小位置
    let chunk_base_offset = UVec3::new(0, 1, 0) * terrain_chunk_info.voxel_num;
    let chunk_coord = chunk_base_offset + UVec3::new(invocation_id.x, 0, invocation_id.z);
    let chunk_voxel_edge_location = terrain_chunk_info.chunk_min_location_size.xyz()
        + Vec3::new(
            chunk_coord.x as f32,
            chunk_coord.y as f32,
            chunk_coord.z as f32,
        ) * terrain_chunk_info.voxel_size;

    let mut seam_voxel_min_location = Vec3::new(0.0, 0.0, 0.0);
    if invocation_id.y == 0 {
        seam_voxel_min_location = chunk_voxel_edge_location - Vec3::new(0.0, voxel_size, 0.0);
    } else {
        seam_voxel_min_location = chunk_voxel_edge_location;
    }

    let mut voxel_vertex_locations = [Vec3::ZERO; 8];
    let mut voxel_vertex_values = [0.0; 8];
    let mut voxel_cross_point_data = [VoxelEdgeCrossPoint::default(); 12];

    compute_voxel_vertices(
        &mut voxel_vertex_locations,
        &mut voxel_vertex_values,
        seam_voxel_min_location,
        voxel_size,
    );
    compute_voxel_cross_points(
        &mut voxel_cross_point_data,
        &voxel_vertex_locations,
        &voxel_vertex_values,
        voxel_size,
    );
    compute_voxel_internal_vertices(
        context,
        &terrain_chunk_info,
        &voxel_cross_point_data,
        invocation_id,
        coord_stride,
        seam_voxel_size,
        1,
    );
}

fn compute_vertices_z_axis(
    context: &mut ComputeVertexContext,
    terrain_chunk_info: &TerrainChunkInfo,
    invocation_id: UVec3,
) {
    // 包括了外部chunk的一个面的体素
    if (invocation_id.x > terrain_chunk_info.voxel_num)
        || (invocation_id.y > terrain_chunk_info.voxel_num)
    {
        return;
    }

    let seam_chunk_size = UVec3::new(
        terrain_chunk_info.voxel_num + 1,
        terrain_chunk_info.voxel_num + 1,
        2,
    );

    // 缝隙chunk的最小位置
    let seam_base_offset = UVec3::new(0, 0, 1) * (terrain_chunk_info.voxel_num - 1);
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
    let lod = context.terrain_chunks_lod[(index * 2 + h_z) as usize][lod_index as usize];

    let lod_scale = 2.0f32.powf(lod as f32).round();
    let coord_stride = UVec3::new(lod_scale as u32, lod_scale as u32, 1);

    if (invocation_id.x % coord_stride.x != 0) || (invocation_id.y % coord_stride.y != 0) {
        return;
    }

    let voxel_size = terrain_chunk_info.voxel_size * lod_scale;

    // 当前体素的最小位置
    let chunk_base_offset = UVec3::new(0, 0, 1) * terrain_chunk_info.voxel_num;
    let chunk_coord = chunk_base_offset + UVec3::new(invocation_id.x, invocation_id.y, 0);
    let chunk_voxel_edge_location = terrain_chunk_info.chunk_min_location_size.xyz()
        + Vec3::new(
            chunk_coord.x as f32,
            chunk_coord.y as f32,
            chunk_coord.z as f32,
        ) * terrain_chunk_info.voxel_size;

    let mut seam_voxel_min_location = Vec3::new(0.0, 0.0, 0.0);
    if invocation_id.z == 0 {
        seam_voxel_min_location = chunk_voxel_edge_location - Vec3::new(0.0, 0.0, voxel_size);
    } else {
        seam_voxel_min_location = chunk_voxel_edge_location;
    }

    let mut voxel_vertex_locations = [Vec3::ZERO; 8];
    let mut voxel_vertex_values = [0.0; 8];
    let mut voxel_cross_point_data = [VoxelEdgeCrossPoint::default(); 12];

    compute_voxel_vertices(
        &mut voxel_vertex_locations,
        &mut voxel_vertex_values,
        seam_voxel_min_location,
        voxel_size,
    );
    compute_voxel_cross_points(
        &mut voxel_cross_point_data,
        &voxel_vertex_locations,
        &voxel_vertex_values,
        voxel_size,
    );
    compute_voxel_internal_vertices(
        context,
        terrain_chunk_info,
        &voxel_cross_point_data,
        invocation_id,
        coord_stride,
        seam_chunk_size,
        2,
    );
}

pub fn compute_x_axis_mesh(
    terrain_chunk_info: &TerrainChunkInfo,
    terrain_chunk_lod: [UVec4; 16],
) -> (ComputeVertexContext, ComputeIndicesContext) {
    let voxel_num = terrain_chunk_info.voxel_num;

    let x = 2;
    let y = voxel_num + 1;
    let z = voxel_num + 1;

    let mut vertex_context = ComputeVertexContext {
        mesh_vertex_locations: vec![Vec4::ZERO; (x * y * z) as usize],
        mesh_vertex_normals: vec![Vec4::ZERO; (x * y * z) as usize],
        mesh_vertex_materials: vec![0; (x * y * z) as usize],
        mesh_vertex_num: 0,
        mesh_vertex_map: vec![0; (x * y * z) as usize],
        terrain_chunks_lod: terrain_chunk_lod,
    };

    for i in 0..x {
        for j in 0..y {
            for k in 0..z {
                let invocation_id = UVec3::new(i, j, k);
                compute_vertices_x_axis(&mut vertex_context, terrain_chunk_info, invocation_id);
            }
        }
    }

    let mut indices_context = ComputeIndicesContext {
        mesh_vertex_locations: vertex_context
            .mesh_vertex_locations
            .iter()
            .map(|x| x.xyz())
            .collect::<Vec<Vec3>>(),
        mesh_vertex_map: vertex_context.mesh_vertex_map.clone(),
        mesh_indices_data: vec![0; (x * y * z * 18) as usize],
        mesh_indices_num: 0,
    };

    for i in 0..x {
        for j in 0..y {
            for k in 0..z {
                let invocation_id = UVec3::new(i, j, k);
                compute_indices_x_axis(&mut indices_context, terrain_chunk_info, invocation_id);
            }
        }
    }

    (vertex_context, indices_context)
}

pub fn compute_y_axis_mesh(
    terrain_chunk_info: &TerrainChunkInfo,
    terrain_chunk_lod: [UVec4; 16],
) -> (ComputeVertexContext, ComputeIndicesContext) {
    let voxel_num = terrain_chunk_info.voxel_num;

    let x = voxel_num + 1;
    let y = 2;
    let z = voxel_num + 1;

    let mut vertex_context = ComputeVertexContext {
        mesh_vertex_locations: vec![Vec4::ZERO; (x * y * z) as usize],
        mesh_vertex_normals: vec![Vec4::ZERO; (x * y * z) as usize],
        mesh_vertex_materials: vec![0; (x * y * z) as usize],
        mesh_vertex_num: 0,
        mesh_vertex_map: vec![0; (x * y * z) as usize],
        terrain_chunks_lod: terrain_chunk_lod,
    };

    for i in 0..x {
        for j in 0..y {
            for k in 0..z {
                let invocation_id = UVec3::new(i, j, k);
                compute_vertices_y_axis(&mut vertex_context, terrain_chunk_info, invocation_id);
            }
        }
    }

    let mut indices_context = ComputeIndicesContext {
        mesh_vertex_locations: vertex_context
            .mesh_vertex_locations
            .iter()
            .map(|x| x.xyz())
            .collect::<Vec<Vec3>>(),
        mesh_vertex_map: vertex_context.mesh_vertex_map.clone(),
        mesh_indices_data: vec![0; (x * y * z * 18) as usize],
        mesh_indices_num: 0,
    };

    for i in 0..x {
        for j in 0..y {
            for k in 0..z {
                let invocation_id = UVec3::new(i, j, k);
                compute_indices_y_axis(&mut indices_context, terrain_chunk_info, invocation_id);
            }
        }
    }

    (vertex_context, indices_context)
}

pub fn compute_z_axis_mesh(
    terrain_chunk_info: &TerrainChunkInfo,
    terrain_chunk_lod: [UVec4; 16],
) -> (ComputeVertexContext, ComputeIndicesContext) {
    let voxel_num = terrain_chunk_info.voxel_num;

    let x = voxel_num + 1;
    let y = voxel_num + 1;
    let z = 2;

    let mut vertex_context = ComputeVertexContext {
        mesh_vertex_locations: vec![Vec4::ZERO; (x * y * z) as usize],
        mesh_vertex_normals: vec![Vec4::ZERO; (x * y * z) as usize],
        mesh_vertex_materials: vec![0; (x * y * z) as usize],
        mesh_vertex_num: 0,
        mesh_vertex_map: vec![0; (x * y * z) as usize],
        terrain_chunks_lod: terrain_chunk_lod,
    };

    for i in 0..x {
        for j in 0..y {
            for k in 0..z {
                let invocation_id = UVec3::new(i, j, k);
                compute_vertices_z_axis(&mut vertex_context, terrain_chunk_info, invocation_id);
            }
        }
    }

    let mut indices_context = ComputeIndicesContext {
        mesh_vertex_locations: vertex_context
            .mesh_vertex_locations
            .iter()
            .map(|x| x.xyz())
            .collect::<Vec<Vec3>>(),
        mesh_vertex_map: vertex_context.mesh_vertex_map.clone(),
        mesh_indices_data: vec![0; (x * y * z * 18) as usize],
        mesh_indices_num: 0,
    };

    for i in 0..x {
        for j in 0..y {
            for k in 0..z {
                let invocation_id = UVec3::new(i, j, k);
                compute_indices_z_axis(&mut indices_context, terrain_chunk_info, invocation_id);
            }
        }
    }

    (vertex_context, indices_context)
}

pub fn compute_seam_mesh(
    terrain_chunk_info: &TerrainChunkInfo,
    terrain_chunk_lod: [UVec4; 16],
) -> (Mesh, Mesh, Mesh) {
    let mut x_axis_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );

    let (vertex, indices) = compute_x_axis_mesh(terrain_chunk_info, terrain_chunk_lod);
    // vertex.log(terrain_chunk_info, 0);
    x_axis_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vertex
            .mesh_vertex_locations
            .iter()
            .map(|x| x.xyz())
            .collect::<Vec<Vec3>>(),
    );
    x_axis_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vertex
            .mesh_vertex_normals
            .iter()
            .map(|x| x.xyz())
            .collect::<Vec<Vec3>>(),
    );
    x_axis_mesh.insert_attribute(MATERIAL_VERTEX_ATTRIBUTE, vertex.mesh_vertex_materials);
    x_axis_mesh.insert_indices(Indices::U32(
        indices
            .mesh_indices_data
            .iter()
            .map(|x| *x as u32)
            .collect(),
    ));

    let mut y_axis_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let (vertex, indices) = compute_y_axis_mesh(terrain_chunk_info, terrain_chunk_lod);
    // vertex.log(terrain_chunk_info, 1);
    y_axis_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vertex
            .mesh_vertex_locations
            .iter()
            .map(|x| x.xyz())
            .collect::<Vec<Vec3>>(),
    );
    y_axis_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vertex
            .mesh_vertex_normals
            .iter()
            .map(|x| x.xyz())
            .collect::<Vec<Vec3>>(),
    );
    y_axis_mesh.insert_attribute(MATERIAL_VERTEX_ATTRIBUTE, vertex.mesh_vertex_materials);
    y_axis_mesh.insert_indices(Indices::U32(
        indices
            .mesh_indices_data
            .iter()
            .map(|x| *x as u32)
            .collect(),
    ));

    let mut z_axis_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let (vertex, indices) = compute_z_axis_mesh(terrain_chunk_info, terrain_chunk_lod);
    // vertex.log(terrain_chunk_info, 2);
    z_axis_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vertex
            .mesh_vertex_locations
            .iter()
            .map(|x| x.xyz())
            .collect::<Vec<Vec3>>(),
    );
    z_axis_mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        vertex
            .mesh_vertex_normals
            .iter()
            .map(|x| x.xyz())
            .collect::<Vec<Vec3>>(),
    );
    z_axis_mesh.insert_attribute(MATERIAL_VERTEX_ATTRIBUTE, vertex.mesh_vertex_materials);
    z_axis_mesh.insert_indices(Indices::U32(
        indices
            .mesh_indices_data
            .iter()
            .map(|x| *x as u32)
            .collect(),
    ));

    (x_axis_mesh, y_axis_mesh, z_axis_mesh)
}
