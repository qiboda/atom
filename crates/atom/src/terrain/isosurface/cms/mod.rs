pub mod bundle;
pub mod extract;
pub mod meshing;
pub mod octree;
pub mod sample;

use std::sync::{Arc, RwLock};

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use futures_lite::future;

use crate::terrain::{
    chunk::{coords::TerrainChunkCoord, TerrainChunk},
    isosurface::{
        cms::{
            bundle::{CMSBundle, CMSTask},
            meshing::mesh::MeshCache,
            octree::octree::Octree,
            sample::surface_sampler::SurfaceSampler,
        },
        IsosurfaceExtractionState,
    },
    materials::TerrainMaterial,
    settings::TerrainSettings,
    TerrainSystemSet,
};

use self::{
    bundle::CMSComponent,
    meshing::mesh::create_mesh,
    octree::octree::{make_octree_structure, mark_transitional_faces},
};

use super::surface::shape_surface::{IsosurfaceContext, ShapeSurface};

#[derive(Default)]
pub struct CMSPlugin {}

impl Plugin for CMSPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, cms_init).add_systems(
            Update,
            (
                cms_update_sampler,
                cms_update_octree,
                cms_update_extrace,
                cms_update_meshing,
                cms_update_create_mesh,
            )
                .before(TerrainSystemSet::GenerateTerrain),
        );
    }
}

fn cms_init(
    mut commands: Commands,
    terrain_settings: Res<TerrainSettings>,
    chunk_coord_query: Query<
        (Entity, &TerrainChunkCoord),
        (Without<CMSComponent>, With<TerrainChunk>),
    >,
) {
    info!("startup_sample_surface: {:?}", chunk_coord_query);
    for (entity, chunk_coord) in chunk_coord_query.iter() {
        let voxel_num = UVec3::splat(terrain_settings.get_chunk_voxel_num());
        let voxel_size = Vec3::splat(terrain_settings.get_chunk_voxel_size());

        let world_offset = Vec3::new(
            chunk_coord.x as f32,
            chunk_coord.y as f32,
            chunk_coord.z as f32,
        ) * voxel_num.as_vec3()
            * voxel_size;

        info!(
            "world_offset: {}, voxel_size: {}, voxel_num: {}",
            world_offset, voxel_size, voxel_num
        );

        commands.entity(entity).insert(CMSBundle {
            cms: CMSComponent {
                mesh_cache: Arc::new(RwLock::new(MeshCache::new())),
                octree: Arc::new(RwLock::new(Octree::default())),
                surface_sampler: Arc::new(RwLock::new(SurfaceSampler::new(
                    world_offset,
                    voxel_size,
                    voxel_num,
                ))),
            },
            task: CMSTask::default(),
        });
    }
}

fn cms_update_sampler(
    mut cms_query: Query<(&mut CMSComponent, &mut CMSTask)>,
    isosurface_context: ResMut<IsosurfaceContext>,
) {
    for (cms_component, mut cms_task) in cms_query.iter_mut() {
        if cms_task.state == IsosurfaceExtractionState::Sample {
            match cms_task.task {
                None => {
                    let thread_pool = AsyncComputeTaskPool::get();

                    let surface_sampler = cms_component.surface_sampler.clone();
                    let shape_surface = isosurface_context.shape_surface.clone();

                    let task = thread_pool.spawn(async move {
                        init_surface_sampler(surface_sampler, shape_surface);
                        info!("init_surface_sampler run over");
                    });
                    cms_task.task = Some(task);
                }
                Some(_) => {
                    info!("cms_task.state == IsosurfaceExtractionState::Sample: task is some");
                    if let Some(_) =
                        future::block_on(future::poll_once(cms_task.task.as_mut().unwrap()))
                    {
                        info!("cms_task.state == IsosurfaceExtractionState::Sample: task is some and ok");
                        cms_task.state = super::IsosurfaceExtractionState::BuildOctree;
                        cms_task.task = None;
                    }
                }
            }
        }
    }
}

fn init_surface_sampler(
    surface_sampler: Arc<RwLock<SurfaceSampler>>,
    shape_surface: Arc<RwLock<ShapeSurface>>,
) {
    let mut surface_sampler = surface_sampler.write().unwrap();
    info_span!("init_surface_sampler");
    info!("init_surface_sampler");
    let offset = surface_sampler.world_offset;
    let size = surface_sampler.voxel_size * surface_sampler.get_sample_size().as_vec3();

    info!(
        "range values: {:?}, {:?}, voxel_size: {:?}",
        offset, size, surface_sampler.voxel_size
    );
    let values =
        shape_surface
            .read()
            .unwrap()
            .get_range_values(offset, size, surface_sampler.voxel_size);
    info!("sample value num: {}", values.len());

    surface_sampler.set_sample_data(values);
}

fn cms_update_octree(
    mut cms_query: Query<(&mut CMSComponent, &mut CMSTask)>,
    isosurface_context: ResMut<IsosurfaceContext>,
    terrain_settings: Res<TerrainSettings>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (cms_component, mut cms_task) in cms_query.iter_mut() {
        if cms_task.state == IsosurfaceExtractionState::BuildOctree {
            match cms_task.task {
                None => {
                    let surface_sampler = cms_component.surface_sampler.clone();
                    let octree = cms_component.octree.clone();
                    let shape_surface = isosurface_context.shape_surface.clone();
                    let terrain_settings = terrain_settings.clone();

                    let task = thread_pool.spawn(async move {
                        make_octree_structure(
                            shape_surface,
                            &terrain_settings,
                            octree.clone(),
                            surface_sampler,
                        );
                        mark_transitional_faces(octree);
                    });
                    cms_task.task = Some(task);
                }
                Some(_) => {
                    info!("cms_task.state == IsosurfaceExtractionState::Octree: task is some");
                    if let Some(_) =
                        future::block_on(future::poll_once(cms_task.task.as_mut().unwrap()))
                    {
                        cms_task.state = super::IsosurfaceExtractionState::Extract;
                        cms_task.task = None;
                    }
                }
            }
        }
    }
}

fn cms_update_extrace(
    mut cms_query: Query<(&mut CMSComponent, &mut CMSTask)>,
    isosurface_context: ResMut<IsosurfaceContext>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (cms_component, mut cms_task) in cms_query.iter_mut() {
        if cms_task.state == IsosurfaceExtractionState::Extract {
            match cms_task.task {
                None => {
                    let surface_sampler = cms_component.surface_sampler.clone();
                    let octree = cms_component.octree.clone();
                    let mesh_cache = cms_component.mesh_cache.clone();
                    let shape_surface = isosurface_context.shape_surface.clone();

                    let task = thread_pool.spawn(async move {
                        let mut octree = octree.write().unwrap();
                        octree.generate_segments(shape_surface, surface_sampler, mesh_cache);
                        octree.edit_transitional_face();
                        octree.trace_comonent();
                    });
                    cms_task.task = Some(task);
                }
                Some(_) => {
                    if let Some(_) =
                        future::block_on(future::poll_once(cms_task.task.as_mut().unwrap()))
                    {
                        cms_task.state = super::IsosurfaceExtractionState::Meshing;
                        cms_task.task = None;
                    }
                }
            }
        }
    }
}

fn cms_update_meshing(mut cms_query: Query<(&mut CMSComponent, &mut CMSTask)>) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (cms_component, mut cms_task) in cms_query.iter_mut() {
        if cms_task.state == IsosurfaceExtractionState::Meshing {
            match cms_task.task {
                None => {
                    let octree = cms_component.octree.clone();
                    let mesh_cache = cms_component.mesh_cache.clone();

                    let task = thread_pool.spawn(async move {
                        let mut octree = octree.write().unwrap();
                        octree.tessellation_traversal(mesh_cache);
                    });
                    cms_task.task = Some(task);
                }
                Some(_) => {
                    if let Some(_) =
                        future::block_on(future::poll_once(cms_task.task.as_mut().unwrap()))
                    {
                        cms_task.state = super::IsosurfaceExtractionState::CreateMesh;
                        cms_task.task = None;
                    }
                }
            }
        }
    }
}

fn cms_update_create_mesh(
    mut commands: Commands,
    mut cms_query: Query<(Entity, &CMSComponent, &mut CMSTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    for (terrain_chunk_entity, cms_component, mut cms_task) in cms_query.iter_mut() {
        if cms_task.state == IsosurfaceExtractionState::CreateMesh {
            let mesh_cache = cms_component.mesh_cache.clone();

            create_mesh(
                &mut commands,
                terrain_chunk_entity,
                mesh_cache,
                &mut meshes,
                &mut materials,
                &asset_server,
            );
            cms_task.state = super::IsosurfaceExtractionState::Done;
        }
    }
}
