use bevy::prelude::*;

use crate::chunk::{chunk_data::TerrainChunkData, TerrainChunk};

use super::{mesh::mesh_info::MeshInfo, state::IsosurfaceState};

#[allow(clippy::type_complexity)]
pub fn update_mesh_lod(
    mut commands: Commands,
    mut query: Query<
        (Entity, &TerrainChunkData, &MeshInfo, &mut IsosurfaceState),
        With<TerrainChunk>,
    >,
) {
    for (entity, terrain_chunk_data, mesh_info, mut state) in query.iter_mut() {
        if *state != IsosurfaceState::UpdateLod {
            continue;
        }

        let lod = terrain_chunk_data.lod;
        if mesh_info.lod != lod {
            if let Some(mut entity_cmds) = commands.get_entity(entity) {
                entity_cmds.remove::<MeshInfo>();
                entity_cmds.despawn_descendants();
            }
            *state = IsosurfaceState::GenMeshInfo;
        }
    }
}
