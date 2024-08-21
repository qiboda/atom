use atom_internal::physical::PhysicalCollisionLayer;
use bevy::prelude::*;

use bevy::pbr::wireframe::WireframeColor;

use bevy::pbr::wireframe::Wireframe;

use avian3d::prelude::*;

use crate::ecology::ecology_set::EcologyMaterials;
use crate::isosurface::dc::gpu_dc::mesh_compute::TerrainChunkMeshDataMainWorldReceiver;
use crate::isosurface::dc::gpu_dc::mesh_compute::TerrainChunkSeamMeshData;
use crate::materials::terrain_mat::TerrainDebugType;

use crate::materials::terrain_mat::TerrainMaterial;

use super::chunk::comp::TerrainChunkAddress;

use super::chunk::comp::TerrainChunkMeshEntities;
use super::chunk::comp::TerrainChunkState;

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
    ecology_materials: Res<EcologyMaterials>,
) {
    loop {
        match receiver.try_recv() {
            Ok(data) => {
                if let Ok((mut state, mut mesh_entities, address)) = query.get_mut(data.entity) {
                    *state = TerrainChunkState::DONE;

                    debug!("receive_terrain_chunk_mesh_data");

                    if let Some(mut main_mesh) = data.main_mesh_data {
                        if let Some(main_mesh_entity) = mesh_entities.main_mesh {
                            commands.entity(main_mesh_entity).despawn_recursive();
                        }

                        if main_mesh.mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_none() {
                            debug!("receive_terrain_chunk_mesh_data main mesh is none");
                            continue;
                        }

                        debug!("receive_terrain_chunk_mesh_data main mesh ok");
                        let material = materials.add(TerrainMaterial {
                            lod: address.0.depth() as u32,
                            debug_type: Some(TerrainDebugType::Normal),
                            // debug_type: None,
                            color_texture: Some(
                                ecology_materials.forest_material.get_albedo_texture(),
                            ),
                            metallic_texture: Some(
                                ecology_materials.forest_material.get_metallic_texture(),
                            ),
                            normal_texture: Some(
                                ecology_materials.forest_material.get_normal_texture(),
                            ),
                            roughness_texture: Some(
                                ecology_materials.forest_material.get_roughness_texture(),
                            ),
                            cull_mode: Some(wgpu::Face::Back),
                        });

                        let mut entity_commands = commands.spawn_empty();

                        // {
                        //     let _span = info_span!("compute main mesh normals").entered();
                        //     main_mesh.mesh.compute_normals();
                        // }

                        entity_commands.insert((
                            Collider::trimesh_from_mesh(&main_mesh.mesh).unwrap(),
                            RigidBody::Static,
                            CollisionLayers::new(
                                PhysicalCollisionLayer::Terrain,
                                [
                                    PhysicalCollisionLayer::Player,
                                    PhysicalCollisionLayer::Enemy,
                                ],
                            ),
                        ));

                        let main_mesh_id = entity_commands
                            .insert((
                                MaterialMeshBundle {
                                    mesh: meshes.add(main_mesh.mesh),
                                    material,
                                    transform: Transform::from_translation(Vec3::splat(0.0)),
                                    visibility: Visibility::Visible,
                                    ..Default::default()
                                },
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
                            TerrainChunkSeamMeshData::GPUMesh(mut gpu_mesh) => {
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
                                    lod: address.0.depth() as u32,
                                    debug_type: Some(TerrainDebugType::Normal),
                                    // debug_type: None,
                                    color_texture: Some(
                                        ecology_materials.forest_material.get_albedo_texture(),
                                    ),
                                    metallic_texture: Some(
                                        ecology_materials.forest_material.get_metallic_texture(),
                                    ),
                                    normal_texture: Some(
                                        ecology_materials.forest_material.get_normal_texture(),
                                    ),
                                    roughness_texture: Some(
                                        ecology_materials.forest_material.get_roughness_texture(),
                                    ),
                                    cull_mode: Some(wgpu::Face::Back),
                                    // cull_mode: None,
                                });

                                let mut entity_commands = commands.spawn_empty();

                                // {
                                //     let _span = info_span!("compute main mesh normals").entered();
                                //     gpu_mesh.seam_mesh.compute_normals();
                                // }

                                entity_commands.insert((
                                    Collider::trimesh_from_mesh(&gpu_mesh.seam_mesh).unwrap(),
                                    RigidBody::Static,
                                    CollisionLayers::new(
                                        PhysicalCollisionLayer::Terrain,
                                        [
                                            PhysicalCollisionLayer::Player,
                                            PhysicalCollisionLayer::Enemy,
                                        ],
                                    ),
                                ));

                                let seam_mesh_id = entity_commands
                                    .insert((
                                        MaterialMeshBundle {
                                            mesh: meshes.add(gpu_mesh.seam_mesh),
                                            material,
                                            transform: Transform::from_translation(Vec3::splat(
                                                0.0,
                                            )),
                                            visibility: Visibility::Visible,
                                            ..Default::default()
                                        },
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
                            TerrainChunkSeamMeshData::CPUMesh(mut cpu_mesh) => {
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
                                    lod: address.0.depth() as u32,
                                    debug_type: Some(TerrainDebugType::Normal),
                                    // debug_type: None,
                                    color_texture: Some(
                                        ecology_materials.forest_material.get_albedo_texture(),
                                    ),
                                    metallic_texture: Some(
                                        ecology_materials.forest_material.get_metallic_texture(),
                                    ),
                                    normal_texture: Some(
                                        ecology_materials.forest_material.get_normal_texture(),
                                    ),
                                    roughness_texture: Some(
                                        ecology_materials.forest_material.get_roughness_texture(),
                                    ),
                                    cull_mode: Some(wgpu::Face::Back),
                                    // cull_mode: None,
                                });

                                let mut entity_commands = commands.spawn_empty();

                                // {
                                //     let _span = info_span!("compute main mesh normals").entered();
                                //     cpu_mesh.seam_mesh.compute_normals();
                                // }

                                entity_commands.insert((
                                    Collider::trimesh_from_mesh(&cpu_mesh.seam_mesh).unwrap(),
                                    RigidBody::Static,
                                    CollisionLayers::new(
                                        PhysicalCollisionLayer::Terrain,
                                        [
                                            PhysicalCollisionLayer::Player,
                                            PhysicalCollisionLayer::Enemy,
                                        ],
                                    ),
                                ));

                                let seam_mesh_id = entity_commands
                                    .insert((
                                        MaterialMeshBundle {
                                            mesh: meshes.add(cpu_mesh.seam_mesh),
                                            material,
                                            transform: Transform::from_translation(Vec3::splat(
                                                0.0,
                                            )),
                                            visibility: Visibility::Visible,
                                            ..Default::default()
                                        },
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
