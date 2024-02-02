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

use std::cell::RefCell;
use std::sync::{Arc, RwLock};

use bevy::math::Vec3A;
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use bundle::{DualContouring, DualContouringBundle, DualContouringTask};
pub use cell_extent::*;
pub use cell_octree::*;
use futures_lite::future;
pub use mesh::*;
pub use sdf::*;
use terrain_player_client::trace::{
    terrain_chunk_trace_span, terrain_trace_triangle, terrain_trace_vertex,
};

use crate::terrain::chunk::chunk_data::TerrainChunkData;
use crate::terrain::chunk::TerrainChunk;
use crate::terrain::ecology::layer::EcologyLayerSampler;
use crate::terrain::materials::terrain::TerrainExtendedMaterial;
use crate::terrain::settings::TerrainSettings;
use crate::terrain::TerrainSystemSet;
use terrain_core::chunk::coords::TerrainChunkCoord;

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
                .chain()
                .in_set(TerrainSystemSet::GenerateTerrain),
        );
    }
}

fn dual_contour_init(
    mut commands: Commands,
    chunk_coord_query: Query<Entity, (Without<DualContouring>, With<TerrainChunk>)>,
) {
    info!("startup_dual_contour_init: {:?}", chunk_coord_query);
    for entity in chunk_coord_query.iter() {
        commands.entity(entity).insert(DualContouringBundle {
            dual_contouring: DualContouring {
                octree: Arc::new(RwLock::new(CellOctree::new(CellId::MAX, vec![]))),
                mesh_cache: Arc::new(RwLock::new(MeshCache::default())),
            },
            dual_contouring_task: DualContouringTask {
                state: DualContourState::BuildingOctree,
                task: None,
            },
        });
    }
}

fn dual_contour_build_octree(
    mut terrain_query: Query<
        (
            &TerrainChunkData,
            &TerrainChunkCoord,
            &mut DualContouring,
            &mut DualContouringTask,
        ),
        With<TerrainChunk>,
    >,
    terrain_setting: Res<TerrainSettings>,
    surface_context: Res<IsosurfaceContext>,
) {
    for (terrain_chunk_data, chunk_coord, dual_contouring, mut dual_contouring_task) in
        terrain_query.iter_mut()
    {
        if dual_contouring_task.state == DualContourState::BuildingOctree {
            info!("dual_contour build tree: {:?}", chunk_coord);
            match dual_contouring_task.task {
                None => {
                    let chunk_size = terrain_setting.chunk_settings.chunk_size;

                    let world_offset = Vec3A::new(
                        chunk_coord.x as f32,
                        chunk_coord.y as f32,
                        chunk_coord.z as f32,
                    ) * chunk_size;

                    let root_cell_extent = CellExtent::new(world_offset, world_offset + chunk_size);

                    let dc = dual_contouring.octree.clone();
                    let surface_shape = surface_context.shape_surface.clone();

                    let chunk_coord_cloned = *chunk_coord;

                    let lod = terrain_chunk_data.lod;
                    info!(
                        "dual contouring build octree: {:?}, lod: {:?}, root_cell_extent: {:?}",
                        chunk_coord_cloned, lod, root_cell_extent
                    );

                    let thread_pool = AsyncComputeTaskPool::get();
                    let task = thread_pool.spawn(async move {
                        let _dc_build =
                            info_span!("dc build", chunk_coord = ?chunk_coord_cloned).entered();

                        let surface_shape = surface_shape.read().unwrap();
                        let mut dc = dc.write().unwrap();
                        dc.build(root_cell_extent, lod, 0.001, 1.0, &surface_shape);
                    });

                    dual_contouring_task.task = Some(task);
                }
                Some(_) => {
                    if future::block_on(future::poll_once(
                        dual_contouring_task.task.as_mut().unwrap(),
                    ))
                    .is_some()
                    {
                        dual_contouring_task.state = DualContourState::DualContouring;
                        dual_contouring_task.task = None;
                    }
                }
            }
        }
    }
}

fn dual_contour_meshing(
    mut dc_query: Query<
        (
            &mut DualContouring,
            &mut DualContouringTask,
            &TerrainChunkCoord,
        ),
        With<TerrainChunk>,
    >,
    surface_context: Res<IsosurfaceContext>,
) {
    for (dual_contouring, mut dual_contouring_task, terrain_chunk_coord) in dc_query.iter_mut() {
        if dual_contouring_task.state == DualContourState::DualContouring {
            info!("dual_contour dual contoring");

            match dual_contouring_task.task {
                None => {
                    let thread_pool = AsyncComputeTaskPool::get();

                    let dc = dual_contouring.octree.clone();
                    let mesh_cache = dual_contouring.mesh_cache.clone();
                    let shape_surface = surface_context.shape_surface.clone();

                    let terrain_chunk_coord = *terrain_chunk_coord;

                    let task = thread_pool.spawn(async move {
                        let mut dc = dc.write().unwrap();
                        let shape_surface = shape_surface.read().unwrap();
                        let mut mesh_cache = mesh_cache.write().unwrap();

                        if !dc.is_valid_octree() {
                            return;
                        }

                        let _dc_dual_contouring =
                            info_span!("dc dual contouring", chunk_coord = ?terrain_chunk_coord)
                                .entered();

                        let mut positions: Vec<Vec3A> = Vec::new();
                        let mut normals = Vec::new();
                        let tri_indices = RefCell::new(Vec::new());

                        let terrain_trace_span = terrain_chunk_trace_span(&terrain_chunk_coord);

                        let terrain_trace_span = terrain_trace_span.enter();

                        dc.dual_contour(
                            |_cell_id, cell| {
                                cell.mesh_vertex_id = positions.len() as MeshVertexId;
                                positions.push(cell.vertex_estimate.into());

                                terrain_trace_vertex(
                                    positions.len(),
                                    (*positions.last().unwrap()).into(),
                                );

                                normals.push(
                                    central_gradient(
                                        &shape_surface,
                                        cell.vertex_estimate.into(),
                                        0.1,
                                    )
                                    .normalize(),
                                );
                            },
                            |q| {
                                tri_indices
                                    .borrow_mut()
                                    .extend_from_slice(&[q[0], q[2], q[1]]);
                                tri_indices
                                    .borrow_mut()
                                    .extend_from_slice(&[q[1], q[2], q[3]]);
                            },
                            |tri| {
                                tri_indices.borrow_mut().extend_from_slice(&tri);
                            },
                        );

                        // Now we need to create the mesh by copying the proper vertices out of the
                        // octree. Since not all vertices will be used, we need to recreate the
                        // vertex IDs based on the new mesh.
                        let all_cells = dc.all_cells();
                        let tri_indices: Vec<_> = tri_indices
                            .into_inner()
                            .into_iter()
                            .map(|i| all_cells[i as usize].mesh_vertex_id)
                            .collect();

                        for tri in tri_indices.chunks_exact(3) {
                            terrain_trace_triangle(
                                tri[0] as usize,
                                tri[1] as usize,
                                tri[2] as usize,
                            );
                        }

                        // repair_sharp_normals(0.75, &mut tri_indices, &mut positions, &mut normals);

                        drop(terrain_trace_span);

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

                    dual_contouring_task.task = Some(task);
                }
                Some(_) => {
                    info!("dual_contour dual contoring waiting task finish");
                    if future::block_on(future::poll_once(
                        dual_contouring_task.task.as_mut().unwrap(),
                    ))
                    .is_some()
                    {
                        dual_contouring_task.state = DualContourState::CreateMesh;
                        dual_contouring_task.task = None;
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
        &DualContouring,
        &mut DualContouringTask,
        &TerrainChunkCoord,
        &EcologyLayerSampler,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainExtendedMaterial>>,
) {
    for (
        terrain_chunk_entity,
        cms_component,
        mut cms_task,
        terrain_chunk_coord,
        ecology_layer_sampler,
    ) in cms_query.iter_mut()
    {
        if cms_task.state == DualContourState::CreateMesh {
            let _dc_create_mesh =
                info_span!("dc create mesh", chunk_coord = ?terrain_chunk_coord).entered();
            info!("create mesh: {:?}", terrain_chunk_coord);
            let mesh_cache = cms_component.mesh_cache.clone();

            create_mesh(
                &mut commands,
                terrain_chunk_entity,
                mesh_cache,
                &mut meshes,
                &mut materials,
                *terrain_chunk_coord,
                ecology_layer_sampler,
            );
            cms_task.state = DualContourState::Done;
        }
    }
}
