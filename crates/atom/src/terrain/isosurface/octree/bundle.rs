use bevy::prelude::Bundle;

use super::{
    cell::{Cell, CellMeshInfo},
    face::Faces,
    octree::{Octree, OctreeCellAddress},
};

#[derive(Bundle)]
pub struct CellBundle {
    pub cell: Cell,
    pub faces: Faces,
    pub cell_mesh_info: CellMeshInfo,
}

#[derive(Bundle, Default)]
pub struct OctreeBundle {
    pub octree: Octree,
    pub octree_cells: OctreeCellAddress,
}
