pub mod mesh_info;

use bevy::{
    math::{bounding::Aabb3d, Vec3A},
    pbr::wireframe::Wireframe,
    prelude::*,
};
use mesh_info::MeshInfo;
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::{
    chunk_mgr::chunk::bundle::TerrainChunk,
    isosurface::{ecology::layer::Sampler, materials::terrain::TerrainMaterial},
};

use super::{
    comp::{IsosurfaceState, TerrainChunkGenerator, TerrainChunkMainMeshCreatedEvent},
    ecology::layer::EcologyLayerSampler,
    materials::terrain::TerrainExtendedMaterial,
};

#[allow(clippy::type_complexity)]
pub fn create_mesh(
    mut commands: Commands,
    chunk_query: Query<(&TerrainChunkCoord, &EcologyLayerSampler), With<TerrainChunk>>,
    mut query: Query<(
        Entity,
        &Parent,
        &MeshInfo,
        &mut IsosurfaceState,
        &TerrainChunkGenerator,
    )>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainExtendedMaterial>>,
    mut event_writer: EventWriter<TerrainChunkMainMeshCreatedEvent>,
) {
    for (chunk_generator_entity, parent, mesh_info, mut state, generator) in query.iter_mut() {
        if *state != IsosurfaceState::CreateMesh {
            continue;
        }

        if mesh_info.is_empty() {
            *state = IsosurfaceState::Done;
            continue;
        }

        let Ok((terrain_chunk_coord, ecology_layer_sampler)) = chunk_query.get(parent.get()) else {
            continue;
        };

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
                        // double_sided: true,
                        // cull_mode: None,
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
                        // double_sided: true,
                        // cull_mode: None,
                        ..default()
                    },
                    extension: TerrainMaterial {
                        base_color: LinearRgba::BLUE,
                    },
                })
            }
        }

        commands.entity(chunk_generator_entity).insert((
            MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(mesh_info)),
                material,
                transform: Transform::from_translation(Vec3::splat(0.0)),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            // RigidBody::Static,
            // Collider::from(&*mesh_cache),
            Wireframe,
        ));

        *state = IsosurfaceState::Done;
        info!("create mesh end");

        event_writer.send(TerrainChunkMainMeshCreatedEvent {
            chunk_entity: parent.get(),
            lod: generator.lod,
        });
    }
}
