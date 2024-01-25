use bevy::{prelude::*, utils::HashMap};

use strum::{EnumCount, IntoEnumIterator};

use crate::terrain::{
    isosurface::{
        octree::{cell::CellType, face::FaceType},
        surface::surface_sampler::SurfaceSampler,
    },
    settings::TerrainSettings,
};

use super::{
    address::CellAddress,
    cell::Cell,
    tables::{SubCellIndex, VertexIndex},
    OctreeBranchPolicy,
};

#[derive(Debug, Component, Default)]
pub struct Octree {
    pub cell_addresses: HashMap<CellAddress, Cell>,
    // todo: points and edges and faces, use address to index.
    // and points num is 8, edges num is 12, faces num is 6. so max bit is 4.
    // some VoxelAddress max bit is 64(usize) - 4 = 60. 60 / 3 = 20. so max octree level is 20.
    // if is 32bit platform, max octree level is (32 - 4) / 3 = 9.
}

pub fn make_octree_structure(
    mut octree: &mut Octree,
    terrain_settings: &TerrainSettings,
    policy: &impl OctreeBranchPolicy,
    mut surface_sampler: &mut impl SurfaceSampler,
) {
    let _make_octree_structure = info_span!("make_octree_structure").entered();
    debug!("make_structure");

    // let task = thread_pool.spawn(async move {
    let c000 = UVec3::new(0, 0, 0);

    let voxel_num = terrain_settings.get_chunk_voxel_num();
    let voxel_num = UVec3::splat(voxel_num);

    // todo: check is branch or leaf cell.....
    let mut address = CellAddress::new();
    address.set(
        CellAddress { raw_address: 1 },
        SubCellIndex::LeftBottomBack,
    );

    let vertex_points = acquire_cell_info(c000, voxel_num);
    let cell = Cell::new(
        CellType::Branch,
        FaceType::BranchFace,
        address,
        vertex_points,
    );

    octree.cell_addresses.insert(address, cell);

    let voxel_num = voxel_num.x >> 1;
    let voxel_num = UVec3::splat(voxel_num);

    subdivide_cell(
        &mut octree,
        address,
        c000,
        voxel_num,
        surface_sampler,
        policy,
    );

    debug!("cell num: {}", octree.cell_addresses.len(),);
}

fn acquire_cell_info(c000: UVec3, voxel_num: UVec3) -> [UVec3; VertexIndex::COUNT] {
    let mut pt_indices = [UVec3::new(0, 0, 0); VertexIndex::COUNT];

    debug_assert!(voxel_num != UVec3::ZERO);

    {
        pt_indices[0] = UVec3::new(c000.x, c000.y, c000.z);
        pt_indices[1] = UVec3::new(c000.x, c000.y, c000.z + voxel_num.z);
        pt_indices[2] = UVec3::new(c000.x, c000.y + voxel_num.y, c000.z);
        pt_indices[3] = UVec3::new(c000.x, c000.y + voxel_num.y, c000.z + voxel_num.z);
        pt_indices[4] = UVec3::new(c000.x + voxel_num.x, c000.y, c000.z);
        pt_indices[5] = UVec3::new(c000.x + voxel_num.x, c000.y, c000.z + voxel_num.z);
        pt_indices[6] = UVec3::new(c000.x + voxel_num.x, c000.y + voxel_num.y, c000.z);
        pt_indices[7] = UVec3::new(
            c000.x + voxel_num.x,
            c000.y + voxel_num.y,
            c000.z + voxel_num.z,
        );
    }

    debug!("pt_indices: {:?}", pt_indices);

    pt_indices
}

fn subdivide_cell(
    mut octree: &mut Octree,
    parent_address: CellAddress,
    parent_c000: UVec3,
    voxel_num: UVec3,
    mut sample_info: &mut impl SurfaceSampler,
    policy: &impl OctreeBranchPolicy,
) {
    // debug!("subdivide_cell: voxel num {}", voxel_num);

    for (i, subcell_index) in SubCellIndex::iter().enumerate() {
        let c000 = UVec3::new(
            parent_c000.x + voxel_num.x * ((i >> 2) & 1) as u32,
            parent_c000.y + voxel_num.y * ((i >> 1) & 1) as u32,
            parent_c000.z + voxel_num.z * (i & 1) as u32,
        );

        let vertex_points = acquire_cell_info(c000, voxel_num);
        let mut address = CellAddress::new();
        address.set(parent_address, subcell_index);

        let mut branch_type = CellType::Branch;
        if policy.check_to_subdivision() {
            debug!("check_for_subdivision can subdivice cell");
            let voxel_num = voxel_num.x >> 1;
            let voxel_num = UVec3::splat(voxel_num);
            if voxel_num.x == 0 {
                branch_type = CellType::Leaf;
            } else {
                subdivide_cell(octree, address, c000, voxel_num, sample_info, policy);
            }
        } else {
            branch_type = CellType::Leaf;
        }

        let face_type = match branch_type {
            CellType::Branch => FaceType::BranchFace,
            CellType::Leaf => FaceType::LeafFace,
        };

        let cell = Cell::new(branch_type, face_type, address, vertex_points);
        octree.cell_addresses.insert(address, cell);

        // debug!(
        //     "subdivide_cell: cell: {:?}",
        //     cell.borrow().get_corner_sample_index()
        // );
        //
    }
}
