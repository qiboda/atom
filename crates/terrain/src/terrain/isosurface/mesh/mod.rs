use std::sync::{Arc, RwLock};

use bevy::{math::Vec3A, pbr::wireframe::Wireframe, prelude::*};
use bevy_xpbd_3d::prelude::{Collider, RigidBody};

use terrain_core::chunk::coords::TerrainChunkCoord;

use crate::terrain::{
    ecology::layer::{EcologyLayerSampler, Sampler},
    isosurface::dc::CellExtent,
    materials::terrain::{TerrainExtendedMaterial, TerrainMaterial},
};

use self::mesh_cache::MeshCache;

pub mod mesh_cache;

pub fn create_mesh(
    commands: &mut Commands,
    terrain_chunk_entity: Entity,
    mesh_cache: Arc<RwLock<MeshCache>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<TerrainExtendedMaterial>>,
    terrain_chunk_coord: TerrainChunkCoord,
    ecology_layer_sampler: &EcologyLayerSampler,
) {
    if let Ok(mesh_cache) = mesh_cache.read() {
        if mesh_cache.is_empty() {
            return;
        }

        let material;
        let ecology_material = ecology_layer_sampler.sample(
            terrain_chunk_coord,
            CellExtent::new(Vec3A::splat(0.0), Vec3A::splat(1.0)),
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
                        base_color: Color::WHITE,
                    },
                })
            }
            None => {
                material = materials.add(TerrainExtendedMaterial {
                    base: StandardMaterial {
                        base_color: Color::BLUE,
                        ..default()
                    },
                    extension: TerrainMaterial {
                        base_color: Color::BLUE,
                    },
                })
            }
        }
        info!(
            "create_mesh: position: {}, indices:{}",
            mesh_cache.get_vertice_positions().len(),
            mesh_cache.get_indices().len(),
        );
        let id = commands
            .spawn((
                MaterialMeshBundle::<TerrainExtendedMaterial> {
                    mesh: meshes.add(Mesh::from(&*mesh_cache)),
                    material,
                    transform: Transform::from_translation(Vec3::splat(0.0)),
                    ..Default::default()
                },
                RigidBody::Static,
                Collider::from(&*mesh_cache),
                Wireframe,
            ))
            .id();

        let mut terrain = commands.get_entity(terrain_chunk_entity).unwrap();
        terrain.add_child(id);
    }
}
