use bevy::prelude::*;

use crate::terrain::chunk::TerrainChunk;

use self::{mesh::MeshCache, tessellation::tessellation_traversal};

use super::IsosurfaceExtractionSet;

pub mod mesh;
pub mod tessellation;
pub mod vertex_index;

pub struct MeshingPlugin;

impl Plugin for MeshingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, add_mesh_cache).add_systems(
            Update,
            tessellation_traversal.in_set(IsosurfaceExtractionSet::Meshing),
        );
    }
}

fn add_mesh_cache(
    mut commands: Commands,
    query: Query<Entity, (Without<MeshCache>, With<TerrainChunk>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(MeshCache::default());
    }
}
