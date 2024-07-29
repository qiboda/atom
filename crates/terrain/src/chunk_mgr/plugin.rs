use avian3d::prelude::{Collider, RigidBody};
use bevy::{
    pbr::wireframe::{Wireframe, WireframeColor},
    prelude::*,
    render::extract_component::ExtractComponentPlugin,
};

use crate::{
    isosurface::{
        dc::gpu_dc::mesh_compute::TerrainChunkMeshDataMainWorldReceiver,
        materials::terrain_mat::{TerrainDebugType, TerrainMaterial},
    },
    TerrainSystemSet,
};

use super::{
    chunk::{
        chunk_aabb::TerrainChunkAabb,
        state::{
            TerrainChunkAddress, TerrainChunkMeshEntities, TerrainChunkSeamLod, TerrainChunkState,
        },
    },
    chunk_loader::TerrainChunkLoaderPlugin,
    chunk_mapper::{
        trigger_chunk_load_event, trigger_chunk_reload_event, trigger_chunk_unload_event,
        TerrainChunkMapper,
    },
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
            .add_systems(PreUpdate, receive_terrain_chunk_mesh_data)
            .add_systems(PreUpdate, update_terrain_chunk_state)
            .observe(trigger_chunk_unload_event)
            .observe(trigger_chunk_reload_event)
            .observe(trigger_chunk_load_event);
        // .add_systems(
        //     Update,
        //     (
        //         update_to_wait_create_seam,
        //         to_create_seam_mesh,
        //         update_create_seam_mesh_over,
        //     )
        //         .chain()
        //         .in_set(TerrainChunkSystemSet::UpdateChunk),
        // );
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

                    if let Some(main_mesh) = data.main_mesh {
                        if let Some(main_mesh_entity) = mesh_entities.main_mesh {
                            commands.entity(main_mesh_entity).despawn_recursive();
                        }

                        if main_mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_none() {
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
                                Collider::trimesh_from_mesh(&main_mesh).unwrap(),
                                MaterialMeshBundle {
                                    mesh: meshes.add(main_mesh),
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

                    if let Some(seam_mesh) = data.seam_mesh {
                        match seam_mesh.axis {
                            0 => {
                                if let Some(mesh_entity) = mesh_entities.right_seam_mesh {
                                    commands.entity(mesh_entity).despawn_recursive();
                                }
                            }
                            1 => {
                                if let Some(mesh_entity) = mesh_entities.top_seam_mesh {
                                    commands.entity(mesh_entity).despawn_recursive();
                                }
                            }
                            2 => {
                                if let Some(mesh_entity) = mesh_entities.front_seam_mesh {
                                    commands.entity(mesh_entity).despawn_recursive();
                                }
                            }
                            _ => {}
                        }

                        if seam_mesh
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
                                Collider::trimesh_from_mesh(&seam_mesh.seam_mesh).unwrap(),
                                MaterialMeshBundle {
                                    mesh: meshes.add(seam_mesh.seam_mesh),
                                    material,
                                    transform: Transform::from_translation(Vec3::splat(0.0)),
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

                        match seam_mesh.axis {
                            0 => {
                                mesh_entities.right_seam_mesh = Some(seam_mesh_id);
                            }
                            1 => {
                                mesh_entities.top_seam_mesh = Some(seam_mesh_id);
                            }
                            2 => {
                                mesh_entities.front_seam_mesh = Some(seam_mesh_id);
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(_) => return,
        }
    }
}
