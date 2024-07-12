use std::{cmp::Ordering, ops::Not};

use bevy::{
    log::{error, info, trace, warn},
    math::{bounding::BoundingVolume, Vec3, Vec3A},
    utils::HashMap,
};
use strum::{EnumCount, IntoEnumIterator};
use terrain_player_client::trace::terrain_trace_vertex;

use crate::isosurface::{
    dc::octree::tables::{EDGE_CELLS_VERTICES, FACE_TO_SUB_EDGES_AXIS_TYPE},
    surface::shape_surface::ShapeSurface,
};

use super::octree::{
    self,
    address::CellAddress,
    cell::{Cell, CellType},
    tables::{
        AxisType, SubCellIndex, FACES_SUBCELLS_NEIGHBOUR_PAIRS, FACES_TO_SUB_EDGES_CELLS,
        SUBCELL_EDGES_NEIGHBOUR_PAIRS, SUBCELL_FACES_NEIGHBOUR_PAIRS, SUBEDGE_CELLS,
    },
    OctreeProxy,
};

pub struct DefaultDualContouringVisiter<'a, 'b> {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3A>,
    pub tri_indices: Vec<u32>,
    pub address_vertex_id_map: HashMap<CellAddress, u32>,
    pub shape_surface: &'a std::sync::RwLockReadGuard<'b, ShapeSurface>,
}

impl<'a, 'b> DefaultDualContouringVisiter<'a, 'b> {
    pub fn new(shape_surface: &'a std::sync::RwLockReadGuard<'b, ShapeSurface>) -> Self {
        Self {
            shape_surface,
            positions: Default::default(),
            normals: Default::default(),
            address_vertex_id_map: Default::default(),
            tri_indices: Default::default(),
        }
    }
}

impl<'a, 'b> DualContouringVisiter for DefaultDualContouringVisiter<'a, 'b> {
    fn visit_cell(&mut self, cell: &Cell) {
        let old = self
            .address_vertex_id_map
            .insert(cell.address, self.positions.len() as u32);
        assert!(old.is_none(), "cell address is duplicated!");
        self.positions.push(cell.vertex_estimate);

        // terrain_trace_vertex(self.positions.len(), *self.positions.last().unwrap());
        self.normals.push(cell.normal_estimate);
        info!(
            "visit cell address::{}, positions:{}",
            cell.address, cell.vertex_estimate
        );
    }

    fn visit_triangle(&mut self, cells: [&octree::cell::Cell; 3]) {
        let vertex_0 = self.address_vertex_id_map.get(&cells[0].address).unwrap();
        let vertex_1 = self.address_vertex_id_map.get(&cells[1].address).unwrap();
        let vertex_2 = self.address_vertex_id_map.get(&cells[2].address).unwrap();
        info!(
            "visit triangle: {}:{}, {}:{}, {}:{}",
            cells[0].address, vertex_0, cells[1].address, vertex_1, cells[2].address, vertex_2
        );
        self.tri_indices
            .extend_from_slice(&[*vertex_0, *vertex_1, *vertex_2]);
    }

    fn visit_quad(&mut self, cells: [&octree::cell::Cell; 4]) {
        let vertex_0 = self.address_vertex_id_map.get(&cells[0].address).unwrap();
        let vertex_1 = self.address_vertex_id_map.get(&cells[1].address).unwrap();
        let vertex_2 = self.address_vertex_id_map.get(&cells[2].address).unwrap();
        let vertex_3 = self.address_vertex_id_map.get(&cells[3].address).unwrap();
        info!(
            "visit quad: {}:{}, {}:{}, {}:{}, {}:{}",
            cells[0].address,
            vertex_0,
            cells[1].address,
            vertex_1,
            cells[2].address,
            vertex_2,
            cells[3].address,
            vertex_3
        );
        self.tri_indices
            .extend_from_slice(&[*vertex_0, *vertex_2, *vertex_1]);
        self.tri_indices
            .extend_from_slice(&[*vertex_1, *vertex_2, *vertex_3]);
    }
}

pub trait DualContouringVisiter {
    fn visit_cell(&mut self, cell: &Cell);

    fn visit_triangle(&mut self, cells: [&Cell; 3]);

    fn visit_quad(&mut self, cells: [&Cell; 4]);
}

pub fn dual_contouring(octree: &OctreeProxy, visiter: &mut impl DualContouringVisiter) {
    let root_address = CellAddress::root();

    if let Some(root_cell) = octree.cell_addresses.get(&root_address) {
        cell_proc(octree, root_cell, visiter);
    }
}

struct FaceCells<'a> {
    pub cells: [&'a Cell; 2],
    pub axis_type: AxisType,
}

struct EdgeCells<'a> {
    // cell的存储顺序是，沿着axis的正方向看去，
    // 2 3
    // 0 1
    // 也就是第0个cell是左下角的cell,
    // 也就是第1个cell是右下角角的cell,
    // 也就是第2个cell是左上角的cell,
    // 也就是第3个cell是右上角的cell,
    pub cells: [&'a Cell; 4],
    pub axis_type: AxisType,
    pub is_dup: [bool; 4],
}

fn cell_proc(octree: &OctreeProxy, parent_cell: &Cell, visiter: &mut impl DualContouringVisiter) {
    info!("cell proc: {:?}", parent_cell.address);

    if parent_cell.cell_type == CellType::Leaf {
        visiter.visit_cell(parent_cell);
        return;
    }

    // 8 subcell in one cell
    let mut subcells = [None; SubCellIndex::COUNT];
    for subcell_index in SubCellIndex::iter() {
        let cell_address = parent_cell.address.get_child_address(subcell_index);
        if let Some(subcell) = octree.cell_addresses.get(&cell_address) {
            cell_proc(octree, subcell, visiter);
            subcells[subcell_index as usize] = Some(subcell);
        }
    }

    // 12 faces within subcell in one cell
    // 作用在于连接八叉树划分下的Cell. 不断细分面，并通过边，将不同Cell内部的点连接起来。
    (0..AxisType::COUNT).for_each(|axis| {
        for (left_cell_index, right_cell_index) in SUBCELL_FACES_NEIGHBOUR_PAIRS[axis] {
            if let (Some(left), Some(right)) = (
                subcells[left_cell_index as usize],
                subcells[right_cell_index as usize],
            ) {
                let face_cells = FaceCells {
                    cells: [left, right],
                    axis_type: AxisType::from_repr(axis).unwrap(),
                };

                info!("cell call face proc");
                face_proc(octree, &face_cells, visiter);
            }
        }
    });

    // 6 edges in one cell
    // 连接八叉树划分下的Cell. 且不断细分边，将不同Cell内部的点连接起来。
    (0..AxisType::COUNT).for_each(|axis| {
        for (cell_index_0, cell_index_1, cell_index_2, cell_index_3) in
            SUBCELL_EDGES_NEIGHBOUR_PAIRS[axis]
        {
            if let (Some(cell_1), Some(cell_2), Some(cell_3), Some(cell_4)) = (
                subcells[cell_index_0 as usize],
                subcells[cell_index_1 as usize],
                subcells[cell_index_2 as usize],
                subcells[cell_index_3 as usize],
            ) {
                let cells = [cell_1, cell_2, cell_3, cell_4];
                info!("cell call edge proc");
                edge_proc(
                    octree,
                    EdgeCells {
                        cells,
                        axis_type: AxisType::from_repr(axis).unwrap(),
                        is_dup: [false; 4],
                    },
                    visiter,
                );
            }
        }
    });
}

/// get child cell or self
fn get_child_cell<'a>(
    octree: &'a OctreeProxy,
    cell: &'a Cell,
    subcell_index: SubCellIndex,
) -> Option<&'a Cell> {
    let cell_address = cell.address.get_child_address(subcell_index);
    match cell.cell_type {
        CellType::Branch => octree.cell_addresses.get(&cell_address),
        CellType::Leaf => Some(cell),
    }
}

fn face_proc(
    octree: &OctreeProxy,
    face_cells: &FaceCells,
    visiter: &mut impl DualContouringVisiter,
) {
    info!(
        "face proc: axix type: {:?}, left cell: {}, right cell: {}",
        face_cells.axis_type, face_cells.cells[0].address, face_cells.cells[1].address
    );
    // face proc
    // 4 faces in one face
    match (face_cells.cells[0].cell_type, face_cells.cells[1].cell_type) {
        (CellType::Branch, CellType::Branch)
        | (CellType::Branch, CellType::Leaf)
        | (CellType::Leaf, CellType::Branch) => {
            let subcell_indices: [(octree::tables::VertexIndex, octree::tables::VertexIndex); 4] =
                FACES_SUBCELLS_NEIGHBOUR_PAIRS[face_cells.axis_type as usize];
            for (subcell_index_0, subcell_index_1) in subcell_indices {
                let subcell_0 = get_child_cell(octree, face_cells.cells[0], subcell_index_0);
                let subcell_1 = get_child_cell(octree, face_cells.cells[1], subcell_index_1);

                if let (Some(subcell_0), Some(subcell_1)) = (subcell_0, subcell_1) {
                    let child_face_cells = FaceCells {
                        cells: [subcell_0, subcell_1],
                        axis_type: face_cells.axis_type,
                    };
                    info!("face call face proc");
                    face_proc(octree, &child_face_cells, visiter);
                }
            }
        }
        (CellType::Leaf, CellType::Leaf) => {
            // 也没有边
            return;
        }
    }

    // 4 edges in one face
    for edge_axis_index in 0..2 {
        for edge_axis_value in 0..2 {
            let [(cell_index_0, axis_value_0), (cell_index_1, axis_value_1), (cell_index_2, axis_value_2), (cell_index_3, axis_value_3)] =
                FACES_TO_SUB_EDGES_CELLS[face_cells.axis_type as usize][edge_axis_index]
                    [edge_axis_value];

            let child_cell_0 = get_child_cell(
                octree,
                face_cells.cells[axis_value_0.bits() as usize],
                cell_index_0,
            );
            let child_cell_1 = get_child_cell(
                octree,
                face_cells.cells[axis_value_1.bits() as usize],
                cell_index_1,
            );
            let child_cell_2 = get_child_cell(
                octree,
                face_cells.cells[axis_value_2.bits() as usize],
                cell_index_2,
            );
            let child_cell_3 = get_child_cell(
                octree,
                face_cells.cells[axis_value_3.bits() as usize],
                cell_index_3,
            );
            if child_cell_0.is_none()
                && child_cell_1.is_some()
                && child_cell_2.is_some()
                && child_cell_3.is_some()
            {
                info!(
                    "mark: 1:{:?}, 2:{:?}, 3:{:?}",
                    child_cell_1.unwrap().coord,
                    child_cell_2.unwrap().coord,
                    child_cell_3.unwrap().coord
                );
            }
            if let (
                Some(child_cell_0),
                Some(child_cell_1),
                Some(child_cell_2),
                Some(child_cell_3),
            ) = (child_cell_0, child_cell_1, child_cell_2, child_cell_3)
            {
                let cells = [child_cell_0, child_cell_1, child_cell_2, child_cell_3];
                let is_dup = [
                    cells[0].address == face_cells.cells[axis_value_0.bits() as usize].address,
                    cells[1].address == face_cells.cells[axis_value_1.bits() as usize].address,
                    cells[2].address == face_cells.cells[axis_value_2.bits() as usize].address,
                    cells[3].address == face_cells.cells[axis_value_3.bits() as usize].address,
                ];
                let edge_axis_type =
                    FACE_TO_SUB_EDGES_AXIS_TYPE[face_cells.axis_type as usize][edge_axis_index];
                info!("face call edge proc");
                edge_proc(
                    octree,
                    EdgeCells {
                        cells,
                        axis_type: edge_axis_type,
                        is_dup: [false; 4],
                    },
                    visiter,
                );
            }
        }
    }
}

fn edge_proc(
    octree: &OctreeProxy,
    edge_cells: EdgeCells,
    visiter: &mut impl DualContouringVisiter,
) {
    info!(
        "edge proc: axix type: {:?}, 0 cell: {}, 1 cell: {}, 2 cell: {}, 3 cell: {}",
        edge_cells.axis_type,
        edge_cells.cells[0].coord,
        edge_cells.cells[1].coord,
        edge_cells.cells[2].coord,
        edge_cells.cells[3].coord,
    );

    if edge_cells
        .cells
        .iter()
        .all(|cell| cell.cell_type == CellType::Leaf)
    {
        visit_leaf_edge(octree, edge_cells, visiter);
        return;
    }

    // get sub edge cells

    for i in 0..2 {
        let [cell_1, cell_2, cell_3, cell_4] = edge_cells.cells;
        let [subcell_index_1, subcell_index_2, subcell_index_3, subcell_index_4] =
            SUBEDGE_CELLS[edge_cells.axis_type as usize][i];

        let child_cell_0 = get_child_cell(octree, cell_1, subcell_index_1);
        let child_cell_1 = get_child_cell(octree, cell_2, subcell_index_2);
        let child_cell_2 = get_child_cell(octree, cell_3, subcell_index_3);
        let child_cell_3 = get_child_cell(octree, cell_4, subcell_index_4);

        info!(
            "edge proc: {:?}, {:?}, {:?}, {:?}",
            child_cell_0, child_cell_1, child_cell_2, child_cell_3
        );
        if let (Some(child_cell_0), Some(child_cell_1), Some(child_cell_2), Some(child_cell_3)) =
            (child_cell_0, child_cell_1, child_cell_2, child_cell_3)
        {
            let sub_edge_cells = [child_cell_0, child_cell_1, child_cell_2, child_cell_3];

            info!("edge call edge proc");
            edge_proc(
                octree,
                EdgeCells {
                    cells: sub_edge_cells,
                    axis_type: edge_cells.axis_type,
                    is_dup: [
                    // sub_edge_cells[0].address == cell_1.address,
                    // sub_edge_cells[1].address == cell_2.address,
                    // sub_edge_cells[2].address == cell_3.address,
                    // sub_edge_cells[3].address == cell_4.address,
                    false; 4
                ],
                },
                visiter,
            );
        }
    }
}

fn visit_leaf_edge(
    _octree: &OctreeProxy,
    edge_cells: EdgeCells,
    visiter: &mut impl DualContouringVisiter,
) {
    assert!(edge_cells
        .cells
        .iter()
        .all(|cell| cell.cell_type == CellType::Leaf));

    info!(
        "leaf edge proc: axix type: {:?}, 0 cell: {}, 1 cell: {}, 2 cell: {}, 3 cell: {}",
        edge_cells.axis_type,
        edge_cells.cells[0].coord,
        edge_cells.cells[1].coord,
        edge_cells.cells[2].coord,
        edge_cells.cells[3].coord
    );

    // Check if this leaf edge is bipolar. We can just check the samples on
    // the smallest cell.
    let mut min_cell_index = 0;
    let mut min_size = f32::MAX;
    for (i, cell) in edge_cells.cells.iter().enumerate() {
        let half_size = cell.aabb.half_size().x;
        if half_size < min_size {
            min_size = half_size;
            min_cell_index = i;
        }
    }
    // Select the edge at the opposite corner of the octant.
    let cell_vertex_indices = EDGE_CELLS_VERTICES[edge_cells.axis_type as usize][min_cell_index];
    let vertex_mats = &edge_cells.cells[min_cell_index].vertices_mat_types;
    let mat0 = vertex_mats[cell_vertex_indices[0] as usize];
    let mat1 = vertex_mats[cell_vertex_indices[1] as usize];

    let flip = match (mat0, mat1) {
        (octree::cell::VoxelMaterialType::Air, octree::cell::VoxelMaterialType::Block) => true,
        (octree::cell::VoxelMaterialType::Block, octree::cell::VoxelMaterialType::Air) => false,
        (octree::cell::VoxelMaterialType::Block, octree::cell::VoxelMaterialType::Block)
        | (octree::cell::VoxelMaterialType::Air, octree::cell::VoxelMaterialType::Air) => {
            // Not a bipolar edge.
            info!(
                "visit leaf edge is not a bipolar edge, mat0: {:?}, mat1: {:?}, axis:{:?}, \
                min_cell_index:{}, vertex_samplers:{:?}->{:?}, {:?}->{:?}, {:?}->{:?}, {:?}->{:?}, {:?}, {:?}",
                mat0,
                mat1,
                edge_cells.axis_type,
                min_cell_index,
                edge_cells.cells[0].coord,
                edge_cells.cells[0].vertices_samples,
                edge_cells.cells[1].coord,
                edge_cells.cells[1].vertices_samples,
                edge_cells.cells[2].coord,
                edge_cells.cells[2].vertices_samples,
                edge_cells.cells[3].coord,
                edge_cells.cells[3].vertices_samples,
                cell_vertex_indices[0],
                cell_vertex_indices[1],
            );
            return;
        }
    };
    info!(
        "cell pos: {:?}, {:?}, {:?}, {:?}, {:?}",
        edge_cells.axis_type,
        edge_cells.cells[0].coord,
        edge_cells.cells[1].coord,
        edge_cells.cells[2].coord,
        edge_cells.cells[3].coord
    );

    // Filter triangles with duplicate vertices (from edges with duplicate
    // cells). Because the triangles must share a diagonal, we know a
    // duplicate can't occur in both triangles. We also know that if any
    // duplicate exists, it will necessarily appear twice around this edge.
    let tris = [[0, 2, 1], [1, 2, 3]];
    let first_tri_num_dups = tris[0]
        .iter()
        .map(|&t| edge_cells.is_dup[t] as u8)
        .sum::<u8>();
    if first_tri_num_dups > 0 {
        // Skip the degenerate triangle.
        let use_tri = if first_tri_num_dups == 1 {
            tris[0]
        } else {
            tris[1]
        };
        if flip {
            let flipped_tri = [use_tri[0], use_tri[2], use_tri[1]];
            visiter.visit_triangle(flipped_tri.map(|i| edge_cells.cells[i]));
        } else {
            visiter.visit_triangle(use_tri.map(|i| edge_cells.cells[i]));
        }
    } else {
        // No degenerate triangles found.
        if flip {
            visiter.visit_quad([
                edge_cells.cells[2],
                edge_cells.cells[3],
                edge_cells.cells[0],
                edge_cells.cells[1],
            ]);
        } else {
            visiter.visit_quad(edge_cells.cells);
        }
    }
}
