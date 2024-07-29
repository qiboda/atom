#define_import_path terrain::voxel_utils

#import terrain::density_field::get_terrain_noise
#import terrain::voxel_type::{VOXEL_MATERIAL_AIR, VOXEL_MATERIAL_BLOCK}

fn get_voxel_vertex_index(voxel_num: u32, x: u32, y: u32, z: u32) -> u32 {
    return x + y * (voxel_num + 1) + z * (voxel_num + 1) * (voxel_num + 1);
}

fn get_voxel_edge_index(voxel_num: u32, x: u32, y: u32, z: u32, edge_index: u32) -> u32 {
    return (x + y * (voxel_num + 1) + z * (voxel_num + 1) * (voxel_num + 1)) * 3 + edge_index;
}

fn get_voxel_index(voxel_num: u32, x: u32, y: u32, z: u32) -> u32 {
    return x + y * voxel_num + z * voxel_num * voxel_num;
}

fn get_voxel_material_type(value: f32) -> u32 {
    if value >= 0 {
        return VOXEL_MATERIAL_AIR;
    }
    return VOXEL_MATERIAL_BLOCK;
}

fn get_voxel_material_type_index(value: f32) -> u32 {
    if value >= 0 {
        return 0u;
    }
    return 1u;
}

// size: terrain_chunk_info.voxel_size;
fn central_gradient(p: vec3f, size: f32) -> vec3f {
    let h = 0.5 * size;
    let x = get_terrain_noise(p + vec3f(h, 0.0, 0.0)) - get_terrain_noise(p - vec3f(h, 0.0, 0.0));
    let y = get_terrain_noise(p + vec3f(0.0, h, 0.0)) - get_terrain_noise(p - vec3f(0.0, h, 0.0));
    let z = get_terrain_noise(p + vec3f(0.0, 0.0, h)) - get_terrain_noise(p - vec3f(0.0, 0.0, h));
    return normalize(vec3f(x, y, z));
}
