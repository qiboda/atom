pub mod address;
pub mod cell;
pub mod tables;
pub mod vertex;

use core::panic;

use bevy::{math::bounding::Aabb3d, prelude::*, utils::HashMap};

use pqef::Quadric;
use strum::{EnumCount, IntoEnumIterator};
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::terrain::{
    isosurface::octree::{cell::CellType, tables::SUBCELL_EDGES_NEIGHBOUR_PAIRS},
    settings::TerrainSettings,
};

use self::tables::{
    AxisType, AXIS_VALUE_SUBCELL_INDICES,
    SUBCELL_FACES_NEIGHBOUR_PAIRS,
};

use {address::CellAddress, cell::Cell, tables::SubCellIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum OctreeSubdivisionResult {
    Subdivided,
    MaxDepth,
    NotSubdivided,
}

pub trait OctreeBranchPolicy {
    fn check_to_subdivision(&self, aabb: Aabb3d) -> OctreeSubdivisionResult;
}

pub trait OctreeSampler {
    fn sampler(&self, loc: Vec3) -> f32;
    fn sampler_split(&self, x: f32, y: f32, z: f32) -> f32;
}

pub trait OctreeContext: OctreeBranchPolicy + OctreeSampler {}

/// octree
/// 1. 细分octree，是否可以细分
/// 2. 计算叶子的qef。
/// 2. 反向进行收缩，节省内容空间。
#[derive(Debug, Component, Default)]
pub struct Octree {
    pub cell_addresses: HashMap<CellAddress, Cell>,
    pub lod: u8,

    pub leaf_cells: Vec<CellAddress>,
}

pub fn make_octree_structure(
    octree: &mut Octree,
    terrain_settings: &TerrainSettings,
    terrain_chunk_coord: TerrainChunkCoord,
    context: &impl OctreeContext,
) {
    let _make_octree_structure: bevy::utils::tracing::span::EnteredSpan =
        info_span!("make_octree_structure").entered();
    debug!("make_structure");

    let root_address = CellAddress::root();

    let chunk_size = terrain_settings.chunk_settings.chunk_size;
    let aabb = Aabb3d::new(Vec3::new(0.0, 0.0, 0.0), Vec3::splat(chunk_size));

    let loc_offset = terrain_chunk_coord * chunk_size;

    if context.check_to_subdivision(aabb) == OctreeSubdivisionResult::Subdivided {
        let cell = Cell::new(CellType::Branch, root_address, aabb);
        subdivide_cell(octree, &cell, loc_offset, context);

        octree.cell_addresses.insert(root_address, cell);
    }

    debug!("cell num: {}", octree.cell_addresses.len(),);
}

fn subdivide_cell(
    octree: &mut Octree,
    parent_cell: &Cell,
    loc_offset: Vec3,
    context: &impl OctreeContext,
) {
    for subcell_index in SubCellIndex::iter() {
        let mut subcell_address = CellAddress::new();
        subcell_address.set(parent_cell.address, subcell_index);

        let subcell_aabb = Cell::get_subcell_aabb(parent_cell.aabb, subcell_index);

        match context.check_to_subdivision(subcell_aabb) {
            OctreeSubdivisionResult::Subdivided => {
                let subcell = Cell::new(CellType::Branch, subcell_address, subcell_aabb);
                subdivide_cell(octree, &subcell, loc_offset, context);
                octree.cell_addresses.insert(subcell_address, subcell);
            }
            OctreeSubdivisionResult::MaxDepth => {
                let mut subcell = Cell::new(CellType::RealLeaf, subcell_address, subcell_aabb);
                for (i, loc) in Cell::get_cell_vertex_locations(subcell_aabb)
                    .iter()
                    .enumerate()
                {
                    subcell.vertices_samples[i] = context.sampler(loc_offset + *loc);
                }

                subcell.estimate_vertex(context, 0.01);

                octree.cell_addresses.insert(subcell_address, subcell);
            }
            OctreeSubdivisionResult::NotSubdivided => {}
        }
    }
}

/// when to collapse:
///     1. all children are real leaf and qef error is less than threshold.
pub fn collapse_octree_leaf_node(octree: &mut Octree) {
    let root_address = CellAddress::root();

    if let Some(root_cell) = octree.cell_addresses.get_mut(&root_address) {
        if root_cell.cell_type == CellType::Branch {
            collapse_octree_leaf_node_recursive(octree, root_address);
        }
    };
}

fn collapse_octree_leaf_node_recursive(octree: &mut Octree, parent_address: CellAddress) -> bool {
    let mut all_children_are_leaf = true;

    let mut parent_regularized_qef = Quadric::default();
    let mut parent_exact_qef = Quadric::default();

    for subcell_index in SubCellIndex::iter() {
        let mut subcell_address = CellAddress::new();
        subcell_address.set(parent_address, subcell_index);

        let mut cell_type = CellType::Branch;
        if subcell_address.get_depth() == octree.lod as usize {
            cell_type = CellType::RealLeaf;
        }

        match cell_type {
            CellType::Branch => {
                all_children_are_leaf =
                    collapse_octree_leaf_node_recursive(octree, subcell_address);
            }
            CellType::RealLeaf | CellType::PseudoLeaf => {
                let subcell = octree
                    .cell_addresses
                    .get_mut(&subcell_address)
                    .expect("subcell is valid!");
                if subcell.qef_error < 0.01 {
                    subcell.cell_type = CellType::PseudoLeaf;
                }
            }
        }

        let subcell = octree
            .cell_addresses
            .get_mut(&subcell_address)
            .expect("subcell is valid!");
        if all_children_are_leaf {
            if let (Some(subcell_regularized_qef), Some(subcell_exact_qef)) =
                (subcell.regularized_qef, subcell.exact_qef)
            {
                parent_regularized_qef += subcell_regularized_qef;
                parent_exact_qef += subcell_exact_qef;
            }
        }
    }

    let parent_cell = octree.cell_addresses.get_mut(&parent_address).unwrap();
    parent_cell.estimate_vertex_with_qef(&parent_regularized_qef, &parent_exact_qef);
    if parent_cell.qef_error < 0.01 {
        parent_cell.cell_type = CellType::PseudoLeaf;
        return true;
    }
    false
}

pub trait DualContouringVisiter {
    fn visit_cell();

    fn visit_triangle();

    fn visit_quad();
}

pub fn dual_contouring(octree: &Octree, visiter: &impl DualContouringVisiter) {
    let root_address = CellAddress::root();

    if let Some(root_cell) = octree.cell_addresses.get(&root_address) {
        cell_proc(octree, root_cell, visiter);
    }
}

struct FaceCells<'a> {
    pub cells: [&'a Cell; 2],
    pub axis_type: AxisType,
}

#[allow(dead_code)]
struct EdgeCells<'a> {
    subcells: [&'a Cell; 4],
}

fn cell_proc(octree: &Octree, parent_cell: &Cell, visiter: &impl DualContouringVisiter) {
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
    (0..AxisType::COUNT).for_each(|axis| {
        for (left_cell_index, right_cell_index) in SUBCELL_FACES_NEIGHBOUR_PAIRS[axis] {
            let face_cells = FaceCells {
                cells: [
                    subcells[left_cell_index as usize].unwrap(),
                    subcells[right_cell_index as usize].unwrap(),
                ],
                axis_type: AxisType::from_bits(axis as u8).unwrap(),
            };

            face_proc(octree, &face_cells, visiter);
        }
    });

    // 6 edges in one cell
    (0..AxisType::COUNT).for_each(|axis| {
        for (cell_index_0, cell_index_1, cell_index_2, cell_index_3) in
            SUBCELL_EDGES_NEIGHBOUR_PAIRS[axis]
        {
            let cells = [
                subcells[cell_index_0 as usize].unwrap(),
                subcells[cell_index_1 as usize].unwrap(),
                subcells[cell_index_2 as usize].unwrap(),
                subcells[cell_index_3 as usize].unwrap(),
            ];
            edge_proc(octree, cells, visiter);
        }
    });
}

/// get child cell or self
fn get_child_cell<'a>(octree: &'a Octree, cell: &'a Cell, subcell_index: SubCellIndex) -> &'a Cell {
    let cell_address = cell.address.get_child_address(subcell_index);
    match octree.cell_addresses.get(&cell_address) {
        Some(v) => v,
        None => cell,
    }
}

fn face_proc(octree: &Octree, face_cells: &FaceCells, _visiter: &impl DualContouringVisiter) {
    // face proc
    // 4 faces in one face
    match (face_cells.cells[0].cell_type, face_cells.cells[1].cell_type) {
        (CellType::Branch, CellType::Branch)
        | (CellType::Branch, CellType::PseudoLeaf | CellType::RealLeaf)
        | (CellType::RealLeaf | CellType::PseudoLeaf, CellType::Branch) => {
            let subcell_indices = AXIS_VALUE_SUBCELL_INDICES[face_cells.axis_type.bits() as usize];
            for (i, subcell_index_0) in subcell_indices[0].into_iter().enumerate() {
                let subcell_index_1 = subcell_indices[1][i];

                let subcell_0 = get_child_cell(octree, face_cells.cells[0], subcell_index_0);
                let subcell_1 = get_child_cell(octree, face_cells.cells[1], subcell_index_1);

                let child_face_cells = FaceCells {
                    cells: [subcell_0, subcell_1],
                    axis_type: face_cells.axis_type,
                };
                face_proc(octree, &child_face_cells, _visiter);
            }
        }
        (CellType::RealLeaf | CellType::PseudoLeaf, CellType::RealLeaf | CellType::PseudoLeaf) => {}
    }

    // 4 edges in one face
}

//
fn edge_proc(_octree: &Octree, _cells: [&Cell; 4], _visiter: &impl DualContouringVisiter) {}
