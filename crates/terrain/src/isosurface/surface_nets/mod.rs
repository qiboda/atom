use std::sync::{Arc, RwLock};

use bevy::{
    math::{bounding::Aabb3d, Vec3A},
    pbr::wireframe::Wireframe,
    prelude::*,
};
use bevy_async_task::AsyncTaskPool;
use fast_surface_nets::{
    ndshape::{RuntimeShape, Shape},
    surface_nets, SurfaceNetsBuffer,
};
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::{
    chunk::TerrainChunk,
    ecology::layer::{EcologyLayerSampler, Sampler},
    materials::terrain::{TerrainExtendedMaterial, TerrainMaterial},
    setting::TerrainSettings,
};

use super::{
    mesh::mesh_info::{MeshInfo, TerrainChunkMesh},
    surface::shape_surface::{IsosurfaceContext, ShapeSurface},
};

#[derive(Debug, Default)]
pub struct SurfaceNetsPlugin;

impl Plugin for SurfaceNetsPlugin {
    fn build(&self, app: &mut App) {
        app.observe(trigger_on_add_terrain_chunk)
            .add_systems(Update, gen_mesh_info)
            .add_systems(Update, create_mesh)
            .add_systems(Update, gen_mesh_info);
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash, Component)]
pub enum SurfaceNetsState {
    GenMeshInfo,
    CreateMesh,
    Done,
}

fn trigger_on_add_terrain_chunk(
    trigger: Trigger<OnAdd, TerrainChunk>,
    mut commands: Commands,
    query: Query<(), (With<TerrainChunk>, Without<SurfaceNetsState>)>,
) {
    let entity = trigger.entity();
    if let Ok(()) = query.get(entity) {
        commands
            .entity(entity)
            .insert(SurfaceNetsState::GenMeshInfo);
    }
}

async fn surface_nets_run_task(
    entity: Entity,
    surface: Arc<RwLock<ShapeSurface>>,
    chunk_size: f32,
    chunk_coord: TerrainChunkCoord,
) -> (Entity, MeshInfo) {
    let offset = chunk_coord * chunk_size;
    let chunk_size = chunk_size as u32 + 2;
    let shape = RuntimeShape::<u32, 3>::new([chunk_size, chunk_size, chunk_size]);

    let mut samples = Vec::with_capacity(shape.size() as usize);
    let surface = surface.read().unwrap();

    for i in 0..shape.size() {
        let loc = offset + Vec3::from_array(shape.delinearize(i).map(|v| v as f32));
        let density = surface.get_value_from_vec(loc);
        samples.push(density);
    }

    info!(
        "surface_nets_run_task: samples: {:?}, chunk_size: {:?}",
        samples.len(),
        chunk_size
    );

    let mut buffer = SurfaceNetsBuffer::default();
    surface_nets(&samples, &shape, [0; 3], [17; 3], &mut buffer);

    let mut mesh_info = MeshInfo::default();
    mesh_info.set_vertice_positions(
        buffer
            .positions
            .into_iter()
            .map(|v| Vec3::new(v[0], v[1], v[2]) + offset)
            .collect(),
    );
    mesh_info.set_vertice_normals(buffer.normals.into_iter().map(|v| v.into()).collect());
    mesh_info.set_indices(buffer.indices);
    (entity, mesh_info)
}

#[allow(clippy::type_complexity)]
pub fn gen_mesh_info(
    mut commands: Commands,
    mut task_pool: AsyncTaskPool<(Entity, MeshInfo)>,
    mut chunk_query: ParamSet<(
        Query<(Entity, &TerrainChunkCoord, &SurfaceNetsState), With<TerrainChunk>>,
        Query<&mut SurfaceNetsState, With<TerrainChunk>>,
    )>,
    settings: Res<TerrainSettings>,
    surface: Res<IsosurfaceContext>,
) {
    if task_pool.is_idle() {
        for (entity, chunk_coord, state) in chunk_query.p0().iter() {
            if state == &SurfaceNetsState::GenMeshInfo {
                let chunk_size = settings.chunk_settings.chunk_size;
                let shape_surface = surface.shape_surface.clone();
                task_pool.spawn(surface_nets_run_task(
                    entity,
                    shape_surface,
                    chunk_size,
                    *chunk_coord,
                ));
            }
        }
    }

    for status in task_pool.iter_poll() {
        match status {
            bevy_async_task::AsyncTaskStatus::Idle => {}
            bevy_async_task::AsyncTaskStatus::Pending => {}
            bevy_async_task::AsyncTaskStatus::Finished((entity, mesh_info)) => {
                commands.entity(entity).insert(mesh_info);
                if let Ok(mut state) = chunk_query.p1().get_mut(entity) {
                    *state = SurfaceNetsState::CreateMesh;
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn create_mesh(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &TerrainChunkCoord,
            &MeshInfo,
            &EcologyLayerSampler,
            &mut SurfaceNetsState,
        ),
        With<TerrainChunk>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainExtendedMaterial>>,
) {
    for (terrain_chunk_entity, terrain_chunk_coord, mesh_info, ecology_layer_sampler, mut state) in
        query.iter_mut()
    {
        if *state != SurfaceNetsState::CreateMesh {
            continue;
        }

        if mesh_info.is_empty() {
            *state = SurfaceNetsState::Done;
            continue;
        }

        let _create_mesh = info_span!("create mesh", chunk_coord = ?terrain_chunk_coord).entered();

        info!(
            "create_mesh: position: {}, indices:{}",
            mesh_info.get_vertice_positions().len(),
            mesh_info.get_indices().len(),
        );

        info!(
            "dual_contour create mesh waiting task finish: {:?}",
            terrain_chunk_coord
        );
        info!("create mesh: {:?}", terrain_chunk_coord);

        let material;
        let ecology_material = ecology_layer_sampler.sample(
            *terrain_chunk_coord,
            Aabb3d::new(Vec3A::splat(0.0), Vec3A::splat(1.0)),
        );

        warn!("create_mesh ok, material: {:?}", ecology_material);
        match &ecology_material {
            Some(ecology_material) => {
                material = materials.add(TerrainExtendedMaterial {
                    base: StandardMaterial {
                        base_color: Color::WHITE,
                        base_color_texture: Some(ecology_material.get_albedo_texture()),
                        perceptual_roughness: 1.0,
                        metallic: 1.0,
                        metallic_roughness_texture: Some(ecology_material.get_roughness_texture()),
                        normal_map_texture: Some(ecology_material.get_normal_texture()),
                        occlusion_texture: Some(ecology_material.get_occlusion_texture()),
                        ..default()
                    },
                    extension: TerrainMaterial {
                        base_color: Color::WHITE.into(),
                    },
                })
            }
            None => {
                material = materials.add(TerrainExtendedMaterial {
                    base: StandardMaterial {
                        base_color: LinearRgba::BLUE.into(),
                        ..default()
                    },
                    extension: TerrainMaterial {
                        base_color: LinearRgba::BLUE,
                    },
                })
            }
        }

        let id = commands
            .spawn((
                MaterialMeshBundle::<TerrainExtendedMaterial> {
                    mesh: meshes.add(Mesh::from(mesh_info)),
                    material,
                    transform: Transform::from_translation(Vec3::splat(0.0)),
                    ..Default::default()
                },
                // RigidBody::Static,
                // Collider::from(&*mesh_cache),
                Wireframe,
                TerrainChunkMesh,
            ))
            .id();

        commands.entity(terrain_chunk_entity).add_child(id);
        *state = SurfaceNetsState::Done;
        info!("create mesh end");
    }
}
