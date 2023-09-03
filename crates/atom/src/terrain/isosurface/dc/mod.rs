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

mod bundle;
mod cell_extent;
mod cell_octree;
mod contour_octree;
mod mesh;
mod sdf;
mod tables;

use std::sync::{Arc, RwLock};

use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bundle::{DualContoring, DualContoringBundle, DualContoringTask};
pub use cell_extent::*;
pub use cell_octree::*;
use futures_lite::future;
pub use mesh::*;
pub use sdf::*;

use crate::terrain::chunk::coords::TerrainChunkCoord;
use crate::terrain::chunk::TerrainChunk;
use crate::terrain::ecology::layer::EcologyLayerSampler;
use crate::terrain::materials::terrain::TerrainMaterial;
use crate::terrain::settings::TerrainSettings;
use crate::terrain::TerrainSystemSet;

use super::mesh::create_mesh;
use super::mesh::mesh_cache::MeshCache;
use super::surface::shape_surface::IsosurfaceContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DualContourState {
    #[default]
    BuildingOctree,
    DualContouring,
    CreateMesh,
    Done,
}

#[derive(Default)]
pub struct DualContourPlugin;

impl Plugin for DualContourPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, dual_contour_init).add_systems(
            Update,
            (
                dual_contour_build_octree,
                dual_contour_meshing,
                dual_contouring_create_mesh,
            )
                .before(TerrainSystemSet::GenerateTerrain),
        );
    }
}

fn dual_contour_init(
    mut commands: Commands,
    chunk_coord_query: Query<Entity, (Without<DualContoring>, With<TerrainChunk>)>,
) {
    // info!("startup_dual_contour_init: {:?}", chunk_coord_query);
    for entity in chunk_coord_query.iter() {
        commands.entity(entity).insert(DualContoringBundle {
            dual_contoring: DualContoring {
                octree: Arc::new(RwLock::new(CellOctree::new(CellId::MAX, vec![]))),
                mesh_cache: Arc::new(RwLock::new(MeshCache::new())),
            },
            dual_contoring_task: DualContoringTask {
                state: DualContourState::BuildingOctree,
                task: None,
            },
        });
    }
}

fn dual_contour_build_octree(
    mut terrain_query: Query<
        (
            &TerrainChunkCoord,
            &mut DualContoring,
            &mut DualContoringTask,
        ),
        With<TerrainChunk>,
    >,
    terrain_setting: Res<TerrainSettings>,
    surface_context: Res<IsosurfaceContext>,
) {
    for (chunk_coord, dual_contoring, mut dual_contoring_task) in terrain_query.iter_mut() {
        if dual_contoring_task.state == DualContourState::BuildingOctree {
            match dual_contoring_task.task {
                None => {
                    let chunk_size = terrain_setting.get_chunk_size();

                    let world_offset = Vec3A::new(
                        chunk_coord.x as f32,
                        chunk_coord.y as f32,
                        chunk_coord.z as f32,
                    ) * chunk_size;
                    let outer_add = Vec3A::new(1.0, 1.0, 1.0);

                    let root_cell_extent = CellExtent::new(
                        world_offset - outer_add,
                        world_offset + chunk_size + outer_add,
                    );

                    let dc = dual_contoring.octree.clone();
                    let surface_shape = surface_context.shape_surface.clone();

                    let thread_pool = AsyncComputeTaskPool::get();
                    let task = thread_pool.spawn(async move {
                        let surface_shape = surface_shape.read().unwrap();
                        let mut dc = dc.write().unwrap();
                        dc.build(root_cell_extent, 7, 0.00001, 0.1, &surface_shape);
                    });

                    dual_contoring_task.task = Some(task);
                }
                Some(_) => {
                    if let Some(_) = future::block_on(future::poll_once(
                        dual_contoring_task.task.as_mut().unwrap(),
                    )) {
                        dual_contoring_task.state = DualContourState::DualContouring;
                        dual_contoring_task.task = None;
                    }
                }
            }
        }
    }
}

fn dual_contour_meshing(
    mut dc_query: Query<(&mut DualContoring, &mut DualContoringTask), With<TerrainChunk>>,
    surface_context: Res<IsosurfaceContext>,
) {
    for (dual_contoring, mut dual_contoring_task) in dc_query.iter_mut() {
        if dual_contoring_task.state == DualContourState::DualContouring {
            match dual_contoring_task.task {
                None => {
                    let thread_pool = AsyncComputeTaskPool::get();

                    let dc = dual_contoring.octree.clone();
                    let mesh_cache = dual_contoring.mesh_cache.clone();
                    let shape_surface = surface_context.shape_surface.clone();

                    let task = thread_pool.spawn(async move {
                        let mut dc = dc.write().unwrap();
                        let shape_surface = shape_surface.read().unwrap();
                        let mut mesh_cache = mesh_cache.write().unwrap();

                        if dc.is_valid_octree() == false {
                            return;
                        }

                        let mut min_leaf_depth = u8::MAX;
                        let mut max_leaf_depth = 0;

                        let mut positions: Vec<Vec3A> = Vec::new();
                        let mut normals = Vec::new();
                        let mut quad_indices = Vec::new();
                        let mut tri_indices = Vec::new();

                        dc.dual_contour(
                            |_cell_id, cell| {
                                min_leaf_depth = min_leaf_depth.min(cell.depth);
                                max_leaf_depth = max_leaf_depth.max(cell.depth);

                                cell.mesh_vertex_id = positions.len() as MeshVertexId;
                                positions.push(cell.vertex_estimate.into());
                                normals.push(
                                    central_gradient(
                                        &shape_surface,
                                        cell.vertex_estimate.into(),
                                        0.001,
                                    )
                                    .normalize(),
                                );
                            },
                            |q| {
                                quad_indices
                                    .extend_from_slice(&[q[0], q[2], q[1], q[1], q[2], q[3]]);
                            },
                            |tri| {
                                tri_indices.extend_from_slice(&tri);
                            },
                        );

                        tri_indices.append(&mut quad_indices);

                        // Now we need to create the mesh by copying the proper vertices out of the
                        // octree. Since not all vertices will be used, we need to recreate the
                        // vertex IDs based on the new mesh.
                        let all_cells = dc.all_cells();
                        let mut tri_indices: Vec<_> = tri_indices
                            .into_iter()
                            .map(|i| all_cells[i as usize].mesh_vertex_id)
                            .collect();

                        repair_sharp_normals(0.95, &mut tri_indices, &mut positions, &mut normals);

                        info!(
                            "tri_indices:{} positions:{} normals:{}",
                            tri_indices.len(),
                            positions.len(),
                            normals.len()
                        );

                        mesh_cache.positions = positions.iter().map(|p| (*p).into()).collect();
                        mesh_cache.normals = normals.iter().map(|n| (*n).into()).collect();
                        mesh_cache.indices = tri_indices;
                    });

                    dual_contoring_task.task = Some(task);
                }
                Some(_) => {
                    if let Some(_) = future::block_on(future::poll_once(
                        dual_contoring_task.task.as_mut().unwrap(),
                    )) {
                        dual_contoring_task.state = DualContourState::CreateMesh;
                        dual_contoring_task.task = None;
                    }
                }
            }
        }
    }
}

pub fn dual_contouring_create_mesh(
    mut commands: Commands,
    mut cms_query: Query<(
        Entity,
        &DualContoring,
        &mut DualContoringTask,
        &TerrainChunkCoord,
        &EcologyLayerSampler,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    for (
        terrain_chunk_entity,
        cms_component,
        mut cms_task,
        _terrain_chunk_coord,
        _ecology_layer_sampler,
    ) in cms_query.iter_mut()
    {
        if cms_task.state == DualContourState::CreateMesh {
            let mesh_cache = cms_component.mesh_cache.clone();

            create_mesh(
                &mut commands,
                terrain_chunk_entity,
                mesh_cache,
                &mut meshes,
                &mut materials,
                &asset_server,
            );
            cms_task.state = DualContourState::Done;
        }
    }
}
