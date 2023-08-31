use std::sync::{Arc, RwLock};

use bevy::prelude::*;
use bevy_xpbd_3d::prelude::{Collider, RigidBody};

use crate::terrain::materials::terrain::TerrainMaterial;

use self::mesh_cache::MeshCache;

pub mod mesh_cache;

pub fn create_mesh(
    commands: &mut Commands,
    terrain_chunk_eneity: Entity,
    mesh_cache: Arc<RwLock<MeshCache>>,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<TerrainMaterial>>,
    asset_server: &ResMut<AssetServer>,
) {
    if let Ok(mesh_cache) = mesh_cache.read() {
        if mesh_cache.is_empty() {
            return;
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
                    material: materials.add(TerrainMaterial {
                        base_color: Color::BLUE,
                        base_color_texture: Some(asset_server.load("output.png")),
                        normal_map_texture: Some(asset_server.load("screenshot_jiumeizi.png")),
                        ..default()
                    }),
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
