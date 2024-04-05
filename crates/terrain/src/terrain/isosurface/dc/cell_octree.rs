use bevy::{prelude::Vec3, reflect::Reflect};
use pqef::Quadric;
use std::ops::Not;
use strum::EnumCount;

use crate::terrain::isosurface::{
    dc::{cell_is_bipolar, estimate_interior_vertex_qef},
    octree::tables::FaceIndex,
    surface::shape_surface::ShapeSurface,
};

use super::{cell_extent::CellExtent, MeshVertexId, NULL_MESH_VERTEX_ID};

#[derive(Debug, Default, Reflect)]
pub struct CellOctree {
    pub(crate) lod: u8,
    pub(crate) root_id: CellId,

    pub(crate) all_cells: Vec<Cell>,

    pub(crate) cell_stack: Vec<CellId>,
    pub(crate) face_stack: Vec<Face>,
    pub(crate) edge_stack: Vec<Edge>,

    pub(crate) seam_cells: [Vec<CellId>; FaceIndex::COUNT],
}

impl CellOctree {
    pub fn new(root_id: CellId, all_cells: Vec<Cell>) -> Self {
        Self {
            root_id,
            all_cells,
            ..Default::default()
        }
    }

    pub fn is_valid_octree(&self) -> bool {
        self.root_id != CellId::MAX
    }

    pub fn all_cells(&self) -> &[Cell] {
        &self.all_cells
    }

    pub(crate) fn clear_stacks(&mut self) {
        self.cell_stack.clear();
        self.face_stack.clear();
        self.edge_stack.clear();
    }

    pub fn build(
        &mut self,
        root_cell: CellExtent,
        max_depth: u8,
        _error_tolerance: f32,
        precision: f32,
        sdf: &ShapeSurface,
    ) {
        let Some(mut root_cell) = Cell::new(root_cell, sdf, 0, max_depth == 0) else {
            return;
        };

        if root_cell.is_leaf {
            let _qef = root_cell.estimate_vertex(sdf, precision);
            self.root_id = 0;
            self.all_cells = vec![root_cell];
            return;
        }

        let (maybe_root_id, _) = self.build_recursive_from_branch(
            max_depth,
            _error_tolerance,
            precision,
            sdf,
            root_cell,
        );

        if let Some(root_id) = maybe_root_id {
            self.root_id = root_id;
        }
    }

    // Recursive because it's easier and slightly more efficient for post-order
    // traversal.
    fn build_recursive_from_branch(
        &mut self,
        max_depth: u8,
        error_tolerance: f32,
        precision: f32,
        sdf: &ShapeSurface,
        mut branch: Cell,
    ) -> (Option<CellId>, VertexState) {
        assert!(!branch.is_leaf);

        // Create all descendant cells.
        let mut sum_descendant_regularized_qef = Quadric::default();
        let mut sum_descendant_exact_qef = Quadric::default();
        let mut child_cell_ids = [None; 8];
        let mut all_nonempty_children_can_merge = true;
        let mut any_nonempty_children = false;
        let mut has_vert = [false; 8];
        let children = branch.get_children(sdf, branch.depth + 1 == max_depth);
        for ((maybe_child, maybe_child_id), has_vert) in children
            .into_iter()
            .zip(&mut child_cell_ids)
            .zip(&mut has_vert)
        {
            let Some(mut child_cell) = maybe_child else {
                continue;
            };

            if child_cell.is_leaf {
                let (regularized_qef, exact_qef) = child_cell.estimate_vertex(sdf, precision);
                sum_descendant_regularized_qef += regularized_qef;
                sum_descendant_exact_qef += exact_qef;

                any_nonempty_children = true;
                let child_id = self.all_cells.len() as CellId;
                self.all_cells.push(child_cell);
                *maybe_child_id = Some(child_id);
            } else {
                let (child_id, child_state) = self.build_recursive_from_branch(
                    max_depth,
                    error_tolerance,
                    precision,
                    sdf,
                    child_cell,
                );
                match child_state {
                    VertexState::EmptySpace => {}
                    VertexState::CannotSimplify => {
                        any_nonempty_children = true;
                        all_nonempty_children_can_merge = false;
                    }
                    VertexState::HasVertex {
                        regularized_qef,
                        exact_qef,
                    } => {
                        any_nonempty_children = true;
                        sum_descendant_regularized_qef += regularized_qef;
                        sum_descendant_exact_qef += exact_qef;
                        *has_vert = true;
                    }
                }
                *maybe_child_id = child_id;
            }
        }

        if any_nonempty_children.not() {
            // Empty branch.
            return (None, VertexState::EmptySpace);
        }

        branch.children = child_cell_ids;

        // Post-order simplification can change branches into pseudo-leaves.

        let mut vertex_state = VertexState::CannotSimplify;
        if all_nonempty_children_can_merge && cell_is_bipolar(&branch.samples) {
            // Branch vertex should be estimated. Only keep if it meets
            // error criterion.
            branch.estimate_vertex_with_qef(
                &sum_descendant_regularized_qef,
                &sum_descendant_exact_qef,
            );
            if branch.qef_error <= error_tolerance {
                // Simplify by choosing a vertex in this branch node.
                branch.is_leaf = true; // pseudo-leaf
                vertex_state = VertexState::HasVertex {
                    regularized_qef: sum_descendant_regularized_qef,
                    exact_qef: sum_descendant_exact_qef,
                };
            }
        }

        if let VertexState::CannotSimplify = vertex_state {
            // Lock child vertices.
            for (child, has_vert) in branch.children.iter().zip(has_vert) {
                if has_vert {
                    let child = child.unwrap();
                    self.all_cells[child as usize].is_leaf = true;
                }
            }
        }

        let branch_id = self.all_cells.len() as CellId;
        self.all_cells.push(branch);

        (Some(branch_id), vertex_state)
    }

    pub fn update_lod(
        &mut self,
        new_lod: u8,
        _error_tolerance: f32,
        precision: f32,
        sdf: &ShapeSurface,
    ) {
        if new_lod == self.lod {
            return;
        }

        if new_lod > self.lod {
            let mut child_cells: Vec<&mut Cell> = self
                .all_cells
                .iter_mut()
                .filter(|cell| cell.is_leaf)
                .collect::<Vec<&mut Cell>>();
            for cell in child_cells.iter_mut() {
                cell.is_leaf = false;
            }

            let child_cells_clone = child_cells
                .iter()
                .map(|cell| (**cell).clone())
                .collect::<Vec<Cell>>();
            for cell in child_cells_clone {
                self.build_recursive_from_branch(new_lod, _error_tolerance, precision, sdf, cell);
            }
        }

        if new_lod < self.lod {
            let mut child_cells: Vec<&mut Cell> = self
                .all_cells
                .iter_mut()
                .filter(|cell| cell.is_leaf)
                .collect::<Vec<&mut Cell>>();
            for cell in child_cells.iter_mut() {
                cell.is_leaf = false;
            }

            let new_child_cells = self
                .all_cells
                .iter_mut()
                .filter(|cell| cell.depth == new_lod)
                // .map(|cell| (*cell).clone())
                .collect::<Vec<&mut Cell>>();
            for cell in new_child_cells {
                cell.is_leaf = true;
            }
        }
    }
}

#[derive(Debug)]
enum VertexState {
    EmptySpace,
    CannotSimplify,
    HasVertex {
        regularized_qef: Quadric,
        exact_qef: Quadric,
    },
}

pub type CellId = u32;

#[derive(Clone, Debug, Reflect)]
pub struct Cell {
    // PERF: replace with a smaller octant identifier; extent should be implicit
    pub extent: CellExtent,

    pub samples: [f32; 8],
    pub children: [Option<CellId>; 8], // PERF: non-zero/non-max?

    // TODO: remove these when meshes are managed separately
    /// We don't use `Vec3A` because it's 16-byte-aligned.
    pub vertex_estimate: Vec3,

    pub mesh_vertex_id: MeshVertexId,
    pub qef_error: f32,

    pub depth: u8,
    pub is_leaf: bool,
}

impl Cell {
    fn new(extent: CellExtent, sdf: &ShapeSurface, depth: u8, is_leaf: bool) -> Option<Self> {
        let cell_positions = extent.corners();
        // PERF: we could pretty easily make 3^3 samples/taps instead of 2^3 * 2^3 when splitting an octant
        // PERF: we could steal 2^3 samples/taps for the parent octant when splitting
        let samples = cell_positions.map(|pos| sdf.get_value(pos.x, pos.y, pos.z));

        if is_leaf && cell_is_bipolar(&samples).not() {
            return None;
        }

        // if branch_empty_check(extent.size().length(), &samples)
        // {
        //     return None;
        // }

        Some(Self {
            extent,
            samples,
            children: [None; 8],
            vertex_estimate: Vec3::ZERO,
            mesh_vertex_id: NULL_MESH_VERTEX_ID,
            qef_error: 0.0,
            is_leaf,
            depth,
        })
    }

    #[inline]
    fn get_children(&self, sdf: &ShapeSurface, is_leaf: bool) -> [Option<Self>; 8] {
        assert!(!self.is_leaf);
        let child_extents = self.extent.split(self.extent.center());
        child_extents.map(|extent| Self::new(extent, sdf, self.depth + 1, is_leaf))
    }

    #[inline]
    fn estimate_vertex(&mut self, sdf: &ShapeSurface, precision: f32) -> (Quadric, Quadric) {
        let (regularized_qef, exact_qef) =
            estimate_interior_vertex_qef(&self.extent, &self.samples, sdf, precision);
        self.estimate_vertex_with_qef(&regularized_qef, &exact_qef);
        (regularized_qef, exact_qef)
    }

    #[inline]
    fn estimate_vertex_with_qef(&mut self, regularized_qef: &Quadric, exact_qef: &Quadric) {
        let p = regularized_qef.minimizer();
        self.qef_error = exact_qef.residual_l2_error(p);
        self.vertex_estimate = p.into();
    }
}

#[derive(Clone, Copy, Debug, Reflect)]
pub(crate) struct Face {
    pub axis: usize,
    pub cells: [CellId; 2],
}

#[derive(Clone, Copy, Debug, Reflect)]
pub(crate) struct Edge {
    pub axis: usize,
    pub cells: [CellId; 4],
    /// True if the corresponding cell appears twice on this edge.
    pub is_duplicate: [bool; 4],
}
