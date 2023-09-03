use std::sync::{Arc, RwLock};

use bevy::{math::Vec3A, prelude::*};
use bevy_xpbd_3d::prelude::{Collider, RigidBody};

use crate::terrain::{
    chunk::coords::TerrainChunkCoord,
    ecology::layer::{EcologyLayerSampler, Sampler},
    isosurface::dc::CellExtent,
    materials::terrain::TerrainMaterial,
};

use self::mesh_cache::MeshCache;

pub mod mesh_cache;

pub fn create_mesh(
    commands: &mut Commands,
    terrain_chunk_eneity: Entity,
    mesh_cache: Arc<RwLock<MeshCache>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<TerrainMaterial>>,
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
        match &ecology_material {
            Some(ecology_material) => {
                material = materials.add(TerrainMaterial {
                    base_color: Color::BLUE,
                    base_color_texture: Some(ecology_material.get_albedo_texture()),
                    normal_map_texture: Some(ecology_material.get_normal_texture()),
                    metallic_texture: Some(ecology_material.get_metallic_texture()),
                    roughness_texture: Some(ecology_material.get_roughness_texture()),
                    occlusion_texture: Some(ecology_material.get_occlusion_texture()),
                })
            }
            None => {
                material = materials.add(TerrainMaterial {
                    base_color: Color::BLUE,
                    ..default()
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
                MaterialMeshBundle::<TerrainMaterial> {
                    mesh: meshes.add(Mesh::from(&*mesh_cache)),
                    material,
                    transform: Transform::from_translation(Vec3::splat(0.0)),
                    ..Default::default()
                },
                RigidBody::Static,
                Collider::from(&*mesh_cache),
            ))
            .id();

        let mut terrian = commands.get_entity(terrain_chunk_eneity).unwrap();
        terrian.add_child(id);
    }
}
