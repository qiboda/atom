use atom_internal::physical::PhysicalCollisionLayer;
use bevy::prelude::*;

use bevy::pbr::wireframe::WireframeColor;

use bevy::pbr::wireframe::Wireframe;

use avian3d::prelude::*;
use wgpu::Face;

use crate::ecology::category::forest::GrassEcologyMaterial;
use crate::isosurface::dc::gpu_dc::mesh_compute::TerrainChunkMeshDataMainWorldReceiver;

use crate::materials::terrain_material::TerrainMaterial;

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
    forest_material: Option<Res<GrassEcologyMaterial>>,
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

                        if forest_material.is_none() {
                            debug!("receive_terrain_chunk_mesh_data forest material is none");
                            continue;
                        }

                        let forest_material = forest_material.as_ref().unwrap();

                        let biomes = main_mesh.get_biomes();

                        info!("biomes: {:?}", biomes);

                        debug!("receive_terrain_chunk_mesh_data main mesh ok");
                        let material = materials.add(TerrainMaterial {
                            lod: address.0.depth(),
                            debug_type: None,
                            // debug_type: None,
                            base_color_texture: forest_material.base_color_texture.clone(),
                            metallic: 0.0,
                            perceptual_roughness: 1.0,
                            metallic_roughness_texture: forest_material
                                .metallic_roughness_texture
                                .clone(),
                            normal_map_texture: forest_material.normal_texture.clone(),
                            occlusion_texture: forest_material.occlusion_texture.clone(),
                            cull_mode: Some(Face::Back),
                            double_sided: false,
                            unlit: false,
                            fog_enabled: true,
                            reflectance: 0.5,
                            attenuation_distance: f32::INFINITY,
                            attenuation_color: Color::WHITE,
                            biome_colors: biomes,
                            ..Default::default()
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

                    if let Some(seam_mesh_data) = data.seam_mesh_data {
                        mesh_entities.seam_mesh.despawn_recursive(&mut commands);

                        if seam_mesh_data
                            .seam_mesh
                            .attribute(Mesh::ATTRIBUTE_POSITION)
                            .is_none()
                        {
                            debug!("receive_terrain_chunk_mesh_data seam mesh is none");
                            continue;
                        }

                        if forest_material.is_none() {
                            debug!("receive_terrain_chunk_mesh_data forest material is none");
                            continue;
                        }

                        let forest_material = forest_material.as_ref().unwrap();

                        let biomes = seam_mesh_data.get_biomes();

                        debug!("receive_terrain_chunk_mesh_data seam mesh ok");
                        let material = materials.add(TerrainMaterial {
                            lod: address.0.depth(),
                            debug_type: None,
                            // debug_type: None,
                            base_color_texture: forest_material.base_color_texture.clone(),
                            metallic: 0.0,
                            perceptual_roughness: 1.0,
                            metallic_roughness_texture: forest_material
                                .metallic_roughness_texture
                                .clone(),
                            normal_map_texture: forest_material.normal_texture.clone(),
                            occlusion_texture: forest_material.occlusion_texture.clone(),
                            cull_mode: Some(Face::Back),
                            double_sided: false,
                            unlit: false,
                            fog_enabled: true,
                            reflectance: 0.5,
                            attenuation_distance: f32::INFINITY,
                            attenuation_color: Color::WHITE,
                            biome_colors: biomes,
                            ..Default::default()
                        });

                        let mut entity_commands = commands.spawn_empty();

                        // {
                        //     let _span = info_span!("compute main mesh normals").entered();
                        //     cpu_mesh.seam_mesh.compute_normals();
                        // }

                        entity_commands.insert((
                            Collider::trimesh_from_mesh(&seam_mesh_data.seam_mesh).unwrap(),
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
                                    mesh: meshes.add(seam_mesh_data.seam_mesh),
                                    material,
                                    transform: Transform::from_translation(Vec3::splat(0.0)),
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
            Err(_) => return,
        }
    }
}
