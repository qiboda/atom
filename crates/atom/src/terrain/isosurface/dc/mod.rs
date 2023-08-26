//! Octree Dual Contouring
//!
//! # References
//!
//! - Tao Ju, Frank Losasso, Scott Schaefer, Joe Warren ["Dual Contouring of
//!   Hermite Data"](https://www.cs.rice.edu/~jwarren/papers/dualcontour.pdf)
//! - Philip Trettner, Leif Kobbelt ["Fast and Robust QEF Minimization using
//!   Probabilistic
//!   Quadrics"](https://www.graphics.rwth-aachen.de/publication/03308/)
//!     - [Reference
//!       implementation](https://github.com/Philip-Trettner/probabilistic-quadrics)
//!
//! # Project Status
//!
//! This is currently just a prototype for understanding the limitations of this
//! technique. My current assessment:
//!
//! ## Pros
//!
//! - can reproduce sharp features from hermite data
//! - built-in octree simplification via QEF
//!
//! ## Cons
//!
//! - requires parameter tuning to avoid artifacts
//! - probably slow? (still need to benchmark)

mod cell_extent;
mod cell_octree;
mod contour_octree;
mod mesh;
mod sdf;
mod tables;

use bevy::math::Vec3A;
use bevy::prelude::*;
pub use cell_extent::*;
pub use cell_octree::*;
pub use mesh::*;
pub use sdf::*;

use crate::terrain::chunk::coords::TerrainChunkCoord;
use crate::terrain::settings::TerrainSettings;

pub struct DualContourPlugin;

impl Plugin for DualContourPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, dual_contour);
    }
}

fn dual_contour(terrain_setting: Res<TerrainSettings>) {
    terrain_setting.get_chunk_size();
}

pub fn extent(chunk_coord: TerrainChunkCoord, chunk_size: f32) {

    // let cell_min = chunk_coord.to_vec3() * chunk_size;
    //
    // let root_cell = CellExtent::new(Vec3A::new(cell_min), Vec3A::new(cell_min + chunk_size));
    //
    // let max_depth = 7;
    // let error_tolerance = 0.00001;
    // let precision = 0.1;
    // let build_t0 = Instant::now();
    // let mut octree =
    //     CellOctree::build(root_cell, max_depth, error_tolerance, precision, field).unwrap();
    // println!("octree build took {} s", build_t0.elapsed().as_secs_f64());
    //
    // let mut min_leaf_depth = u8::MAX;
    // let mut max_leaf_depth = 0;
    //
    // let mut positions: Vec<Vec3A> = Vec::new();
    // let mut normals = Vec::new();
    // let mut quad_indices = Vec::new();
    // let mut tri_indices = Vec::new();
    // let contour_t0 = Instant::now();
    //
    // octree.dual_contour(
    //     |_cell_id, cell| {
    //         min_leaf_depth = min_leaf_depth.min(cell.depth);
    //         max_leaf_depth = max_leaf_depth.max(cell.depth);
    //
    //         cell.mesh_vertex_id = positions.len() as MeshVertexId;
    //         positions.push(cell.vertex_estimate.into());
    //         normals.push(central_gradient(&field, cell.vertex_estimate.into(), 0.001).normalize());
    //     },
    //     |q| {
    //         quad_indices.extend_from_slice(&[q[0], q[2], q[1], q[1], q[2], q[3]]);
    //     },
    //     |tri| {
    //         tri_indices.extend_from_slice(&tri);
    //     },
    // );
    // println!("dual contour took {} s", contour_t0.elapsed().as_secs_f64());
    // println!("depth = {min_leaf_depth}..={max_leaf_depth}");
    //
    // tri_indices.append(&mut quad_indices);
    //
    // // Now we need to create the mesh by copying the proper vertices out of the
    // // octree. Since not all vertices will be used, we need to recreate the
    // // vertex IDs based on the new mesh.
    // let all_cells = octree.all_cells();
    // let mut tri_indices: Vec<_> = tri_indices
    //     .into_iter()
    //     .map(|i| all_cells[i as usize].mesh_vertex_id)
    //     .collect();
    //
    // repair_sharp_normals(0.95, &mut tri_indices, &mut positions, &mut normals);
    //
    // println!("# isosurface vertices = {}", positions.len());
    // println!("# isosurface triangles = {}", tri_indices.len() / 3);
    //
    // let mut isomesh = Mesh::new(PrimitiveTopology::TriangleList);
    // isomesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions.clone());
    // isomesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
    // isomesh.set_indices(Some(Indices::U32(tri_indices)));
    // let isomesh = meshes.add(isomesh);
}
