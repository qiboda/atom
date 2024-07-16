use bevy::prelude::*;

use crate::isosurface::comp::{TerrainChunkGenerator, TerrainChunkMainMeshCreatedEvent};

use super::chunk::{bundle::TerrainChunk, state::TerrainChunkState};

pub fn read_main_mesh_created_event(
    mut event_reader: EventReader<TerrainChunkMainMeshCreatedEvent>,
    mut query: Query<(&Children, &mut TerrainChunkState), With<TerrainChunk>>,
    mut generator_query: Query<(&mut Visibility, &TerrainChunkGenerator)>,
) {
    for event in event_reader.read() {
        if let Ok((children, mut chunk_state)) = query.get_mut(event.chunk_entity) {
            if let TerrainChunkState::CreateMainMesh(lod) = *chunk_state {
                if lod == event.lod {
                    *chunk_state = TerrainChunkState::CreateMainMeshOver;

                    info!(
                        "read_main_mesh_created_event, children: NUM:{}",
                        children.len()
                    );
                    for child in children.iter() {
                        if let Ok((mut visibility, generator)) = generator_query.get_mut(*child) {
                            if generator.lod == lod {
                                *visibility = Visibility::Visible;
                            } else {
                                *visibility = Visibility::Hidden;
                            }
                        } else {
                            info!("read_main_mesh_created_event, get_generator fail",);
                        }
                    }
                }
            }
        }
    }
}
