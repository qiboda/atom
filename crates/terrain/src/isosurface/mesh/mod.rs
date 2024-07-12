pub mod mesh_info;

use bevy::{
    math::{bounding::Aabb3d, Vec3A},
    pbr::wireframe::Wireframe,
    prelude::*,
};
use bevy_async_task::AsyncTaskPool;
use mesh_info::{MeshInfo, TerrainChunkMesh};
use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::{
    chunk::{chunk_data::TerrainChunkData, TerrainChunk},
    ecology::layer::{EcologyLayerSampler, Sampler},
    materials::{
        terrain::{TerrainExtendedMaterial, TerrainMaterial},
        terrain_debug::TerrainDebugMaterial,
    },
    setting::TerrainSetting,
};

use super::{state::IsosurfaceState, surface::shape_surface::IsosurfaceContext};

#[allow(clippy::type_complexity)]
pub fn create_mesh(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &TerrainChunkCoord,
            &TerrainChunkData,
            &MeshInfo,
            &EcologyLayerSampler,
            &mut IsosurfaceState,
        ),
        With<TerrainChunk>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<TerrainExtendedMaterial>>,
    mut debug_materials: ResMut<Assets<TerrainDebugMaterial>>,
) {
    for (
        terrain_chunk_entity,
        terrain_chunk_coord,
        terrain_chunk_data,
        mesh_info,
        ecology_layer_sampler,
        mut state,
    ) in query.iter_mut()
    {
        if *state != IsosurfaceState::CreateMesh {
            continue;
        }

        if mesh_info.is_empty() {
            *state = IsosurfaceState::Done;
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

        // let mut debug_material = TerrainDebugMaterial {
        //     color: match terrain_chunk_data.lod {
        //         0 => LinearRgba::RED,
        //         1 => LinearRgba::GREEN,
        //         2 => LinearRgba::BLUE,
        //         4 => LinearRgba::RED,
        //         5 => LinearRgba::GREEN,
        //         _ => LinearRgba::WHITE,
        //     },
        // };
        // let material = debug_materials.add(debug_material);

        let id = commands
            .spawn((
                MaterialMeshBundle {
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

        if let Some(mut entity_cmds) = commands.get_entity(terrain_chunk_entity) {
            entity_cmds.add_child(id);
        }
        *state = IsosurfaceState::UpdateLod;
        info!("create mesh end");
    }
}
