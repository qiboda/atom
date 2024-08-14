use avian3d::prelude::{Collider, RigidBody};
use bevy::{
    pbr::wireframe::{Wireframe, WireframeColor},
    prelude::*,
    render::{extract_component::ExtractComponentPlugin, extract_resource::ExtractResourcePlugin},
};

use crate::{
    isosurface::{
        dc::gpu_dc::mesh_compute::{
            TerrainChunkMeshDataMainWorldReceiver, TerrainChunkSeamMeshData,
        },
        materials::terrain_mat::{TerrainDebugType, TerrainMaterial},
    },
    TerrainSystemSet,
};

use super::{
    chunk::{
        chunk_aabb::TerrainChunkAabb,
        state::{
            TerrainChunkAddress, TerrainChunkBorderVertices, TerrainChunkMeshEntities,
            TerrainChunkNeighborLodNodes, TerrainChunkSeamLod, TerrainChunkState,
        },
    },
    chunk_event::{
        trigger_chunk_load_event, trigger_chunk_reload_event, trigger_chunk_unload_event,
    },
    chunk_loader::TerrainChunkLoaderPlugin,
    chunk_mapper::TerrainChunkMapper,
    TerrainChunkSystemSet,
};

#[derive(Default, Debug)]
pub struct TerrainChunkPlugin;

impl Plugin for TerrainChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainChunkMapper>()
            .configure_sets(
                Update,
                (
                    TerrainChunkSystemSet::UpdateLoader,
                    TerrainChunkSystemSet::UpdateChunk,
                )
                    .chain()
                    .in_set(TerrainSystemSet::UpdateChunk),
            )
            .add_plugins(TerrainChunkLoaderPlugin)
            .add_plugins(ExtractComponentPlugin::<TerrainChunkState>::default())
            .add_plugins(ExtractComponentPlugin::<TerrainChunkAddress>::default())
            .add_plugins(ExtractComponentPlugin::<TerrainChunkAabb>::default())
            .add_plugins(ExtractComponentPlugin::<TerrainChunkSeamLod>::default())
            .add_plugins(ExtractComponentPlugin::<TerrainChunkNeighborLodNodes>::default())
            .add_plugins(ExtractComponentPlugin::<TerrainChunkBorderVertices>::default())
            .add_plugins(ExtractResourcePlugin::<TerrainChunkMapper>::default())
            .add_systems(PreUpdate, receive_terrain_chunk_mesh_data)
            .add_systems(PreUpdate, update_terrain_chunk_state)
            .observe(trigger_chunk_unload_event)
            .observe(trigger_chunk_reload_event)
            .observe(trigger_chunk_load_event);
    }
}

pub fn update_terrain_chunk_state(mut query: Query<&mut TerrainChunkState>) {
    for mut state in query.iter_mut() {
        if *state != TerrainChunkState::DONE {
            *state = TerrainChunkState::DONE;
        }
    }
}

pub fn receive_terrain_chunk_mesh_data(
    mut commands: Commands,
    receiver: Res<TerrainChunkMeshDataMainWorldReceiver>,
    mut query: Query<(
        &mut TerrainChunkState,
        &mut TerrainChunkMeshEntities,
        &TerrainChunkAddress,
    )>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    loop {
        match receiver.try_recv() {
            Ok(data) => {
                if let Ok((mut state, mut mesh_entities, address)) = query.get_mut(data.entity) {
                    *state = TerrainChunkState::DONE;

                    debug!("receive_terrain_chunk_mesh_data");

                    if let Some(main_mesh) = data.main_mesh_data {
                        if let Some(main_mesh_entity) = mesh_entities.main_mesh {
                            commands.entity(main_mesh_entity).despawn_recursive();
                        }

                        if main_mesh.mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_none() {
                            debug!("receive_terrain_chunk_mesh_data main mesh is none");
                            continue;
                        }

                        debug!("receive_terrain_chunk_mesh_data main mesh ok");
                        let material = materials.add(TerrainMaterial {
                            lod: address.0.level() as u32,
                            debug_type: Some(TerrainDebugType::Color),
                            color_texture: None,
                            metallic_texture: None,
                            normal_texture: None,
                            roughness_texture: None,
                            cull_mode: Some(wgpu::Face::Back),
                        });

                        let main_mesh_id = commands
                            .spawn((
                                Collider::trimesh_from_mesh(&main_mesh.mesh).unwrap(),
                                MaterialMeshBundle {
                                    mesh: meshes.add(main_mesh.mesh),
                                    material,
                                    transform: Transform::from_translation(Vec3::splat(0.0)),
                                    visibility: Visibility::Visible,
                                    ..Default::default()
                                },
                                RigidBody::Static,
                                Wireframe,
                                WireframeColor {
                                    color: LinearRgba::BLACK.into(),
                                },
                            ))
                            .set_parent(data.entity)
                            .id();
                        mesh_entities.main_mesh = Some(main_mesh_id);
                    }

                    if let Some(seam_mesh) = data.seam_mesh_data {
                        match seam_mesh {
                            TerrainChunkSeamMeshData::GPUMesh(gpu_mesh) => {
                                mesh_entities.seam_mesh.despawn_recursive(&mut commands);

                                if gpu_mesh
                                    .seam_mesh
                                    .attribute(Mesh::ATTRIBUTE_POSITION)
                                    .is_none()
                                {
                                    debug!("receive_terrain_chunk_mesh_data seam mesh is none");
                                    continue;
                                }

                                debug!("receive_terrain_chunk_mesh_data seam mesh ok");
                                let material = materials.add(TerrainMaterial {
                                    lod: address.0.level() as u32,
                                    debug_type: Some(TerrainDebugType::Color),
                                    color_texture: None,
                                    metallic_texture: None,
                                    normal_texture: None,
                                    roughness_texture: None,
                                    cull_mode: Some(wgpu::Face::Back),
                                    // cull_mode: None,
                                });

                                let seam_mesh_id = commands
                                    .spawn((
                                        Collider::trimesh_from_mesh(&gpu_mesh.seam_mesh).unwrap(),
                                        MaterialMeshBundle {
                                            mesh: meshes.add(gpu_mesh.seam_mesh),
                                            material,
                                            transform: Transform::from_translation(Vec3::splat(
                                                0.0,
                                            )),
                                            visibility: Visibility::Visible,
                                            ..Default::default()
                                        },
                                        RigidBody::Static,
                                        Wireframe,
                                        WireframeColor {
                                            color: LinearRgba::WHITE.into(),
                                        },
                                    ))
                                    .set_parent(data.entity)
                                    .id();

                                mesh_entities
                                    .seam_mesh
                                    .set_gpu_seam_mesh(seam_mesh_id, gpu_mesh.axis);
                            }
                            TerrainChunkSeamMeshData::CPUMesh(cpu_mesh) => {
                                mesh_entities.seam_mesh.despawn_recursive(&mut commands);

                                if cpu_mesh
                                    .seam_mesh
                                    .attribute(Mesh::ATTRIBUTE_POSITION)
                                    .is_none()
                                {
                                    debug!("receive_terrain_chunk_mesh_data seam mesh is none");
                                    continue;
                                }

                                debug!("receive_terrain_chunk_mesh_data seam mesh ok");
                                let material = materials.add(TerrainMaterial {
                                    lod: address.0.level() as u32,
                                    debug_type: Some(TerrainDebugType::Color),
                                    color_texture: None,
                                    metallic_texture: None,
                                    normal_texture: None,
                                    roughness_texture: None,
                                    cull_mode: Some(wgpu::Face::Back),
                                    // cull_mode: None,
                                });

                                let seam_mesh_id = commands
                                    .spawn((
                                        Collider::trimesh_from_mesh(&cpu_mesh.seam_mesh).unwrap(),
                                        MaterialMeshBundle {
                                            mesh: meshes.add(cpu_mesh.seam_mesh),
                                            material,
                                            transform: Transform::from_translation(Vec3::splat(
                                                0.0,
                                            )),
                                            visibility: Visibility::Visible,
                                            ..Default::default()
                                        },
                                        RigidBody::Static,
                                        Wireframe,
                                        WireframeColor {
                                            color: LinearRgba::WHITE.into(),
                                        },
                                    ))
                                    .set_parent(data.entity)
                                    .id();

                                mesh_entities.seam_mesh.set_cpu_seam_mesh(seam_mesh_id);
                            }
                        }
                    }
                }
            }
            Err(_) => return,
        }
    }
}
