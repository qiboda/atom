pub mod mesh_info;

use avian3d::{collision::Collider, prelude::RigidBody};
use bevy::{
    math::{bounding::Aabb3d, Vec3A},
    pbr::wireframe::{Wireframe, WireframeColor},
    prelude::*,
};
use mesh_info::MeshInfo;
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::{
    chunk_mgr::chunk::{bundle::TerrainChunk, chunk_lod::TerrainChunkLod},
    isosurface::{ecology::layer::Sampler, materials::terrain_mat::TerrainMaterial},
};

use super::{
    comp::{MainMeshState, SeamMeshState, TerrainChunkMainGenerator, TerrainChunkSeamGenerator},
    ecology::layer::EcologyLayerSampler,
};

#[allow(clippy::type_complexity)]
pub fn create_main_mesh(
    mut commands: Commands,
    chunk_query: Query<(&TerrainChunkCoord, &EcologyLayerSampler), With<TerrainChunk>>,
    mut query: Query<(
        Entity,
        &Parent,
        &MeshInfo,
        &mut MainMeshState,
        &TerrainChunkMainGenerator,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    for (chunk_generator_entity, parent, mesh_info, mut state, generator) in query.iter_mut() {
        if *state != MainMeshState::CreateMesh {
            continue;
        }

        if mesh_info.is_empty() {
            *state = MainMeshState::Done;
            continue;
        }

        let Ok((terrain_chunk_coord, ecology_layer_sampler)) = chunk_query.get(parent.get()) else {
            *state = MainMeshState::Done;
            continue;
        };

        let _create_mesh =
            info_span!("main mesh create", %terrain_chunk_coord, generator.lod).entered();

        debug!(
            "create main mesh: position: {}, indices:{}",
            mesh_info.get_vertex_positions().len(),
            mesh_info.get_indices().len(),
        );

        let material;
        let ecology_material = ecology_layer_sampler.sample(
            *terrain_chunk_coord,
            Aabb3d::new(Vec3A::splat(0.0), Vec3A::splat(1.0)),
        );

        match &ecology_material {
            Some(ecology_material) => {
                material = materials.add(TerrainMaterial {
                    color_texture: Some(ecology_material.get_albedo_texture()),
                    metallic_texture: Some(ecology_material.get_metallic_texture()),
                    normal_texture: Some(ecology_material.get_normal_texture()),
                    roughness_texture: Some(ecology_material.get_roughness_texture()),
                })
            }
            None => {
                material = materials.add(TerrainMaterial {
                    color_texture: None,
                    metallic_texture: None,
                    normal_texture: None,
                    roughness_texture: None,
                })
            }
        }

        commands.entity(chunk_generator_entity).insert((
            MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(mesh_info)),
                material,
                transform: Transform::from_translation(Vec3::splat(0.0)),
                visibility: Visibility::Visible,
                ..Default::default()
            },
            RigidBody::Static,
            Collider::from(mesh_info),
            Wireframe,
            WireframeColor {
                color: LinearRgba::GREEN.into(),
            },
        ));

        *state = MainMeshState::Done;
        info!("create main mesh end: {}", terrain_chunk_coord);
    }
}

#[allow(clippy::type_complexity)]
pub fn create_seam_mesh(
    mut commands: Commands,
    chunk_query: Query<
        (&TerrainChunkCoord, &TerrainChunkLod, &EcologyLayerSampler),
        With<TerrainChunk>,
    >,
    mut query: Query<
        (
            Entity,
            &Parent,
            &MeshInfo,
            &mut SeamMeshState,
            Option<&Handle<Mesh>>,
        ),
        With<TerrainChunkSeamGenerator>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainMaterial>>,
) {
    for (seam_generator_entity, parent, mesh_info, mut state, mesh_handle) in query.iter_mut() {
        if *state != SeamMeshState::CreateMesh {
            continue;
        }

        if mesh_info.is_empty() {
            *state = SeamMeshState::Done;
            continue;
        }

        let Ok((terrain_chunk_coord, chunk_lod, ecology_layer_sampler)) =
            chunk_query.get(parent.get())
        else {
            *state = SeamMeshState::Done;
            continue;
        };

        let _create_mesh =
            info_span!("seam mesh create", %terrain_chunk_coord, lod = chunk_lod.get_lod())
                .entered();

        debug!(
            "create seam mesh: position: {}, indices:{}",
            mesh_info.get_vertex_positions().len(),
            mesh_info.get_indices().len(),
        );

        let material;
        let ecology_material = ecology_layer_sampler.sample(
            *terrain_chunk_coord,
            Aabb3d::new(Vec3A::splat(0.0), Vec3A::splat(1.0)),
        );

        match &ecology_material {
            Some(ecology_material) => {
                // TODO: 缓存材质，避免重复创建
                material = materials.add(TerrainMaterial {
                    color_texture: Some(ecology_material.get_albedo_texture()),
                    metallic_texture: Some(ecology_material.get_metallic_texture()),
                    normal_texture: Some(ecology_material.get_normal_texture()),
                    roughness_texture: Some(ecology_material.get_roughness_texture()),
                })
            }
            None => {
                material = materials.add(TerrainMaterial {
                    color_texture: None,
                    metallic_texture: None,
                    normal_texture: None,
                    roughness_texture: None,
                })
            }
        }

        match mesh_handle {
            Some(handle) => {
                let mesh = meshes.get_mut(handle.id()).unwrap();
                *mesh = Mesh::from(mesh_info);
                warn!("replace mesh");
            }
            None => {
                commands.entity(seam_generator_entity).insert((
                    MaterialMeshBundle {
                        mesh: meshes.add(Mesh::from(mesh_info)),
                        material,
                        transform: Transform::from_translation(Vec3::splat(0.0)),
                        visibility: Visibility::Visible,
                        ..Default::default()
                    },
                    RigidBody::Static,
                    Collider::from(mesh_info),
                    Wireframe,
                    WireframeColor {
                        color: LinearRgba::RED.into(),
                    },
                ));
            }
        }

        *state = SeamMeshState::Done;
        info!("create seam mesh end: {}", terrain_chunk_coord);
    }
}
