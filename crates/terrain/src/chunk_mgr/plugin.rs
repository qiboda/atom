use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponentPlugin, extract_resource::ExtractResourcePlugin},
};

use crate::TerrainSystemSet;

use super::{
    chunk::comp::{
        TerrainChunkAabb, TerrainChunkAddress, TerrainChunkBorderVertices,
        TerrainChunkNeighborLodNodes, TerrainChunkSeamLod, TerrainChunkState,
    },
    chunk_event::{
        trigger_chunk_load_event, trigger_chunk_reload_event, trigger_chunk_unload_event,
    },
    chunk_loader::TerrainChunkLoaderPlugin,
    chunk_mapper::TerrainChunkMapper,
    chunk_mesh::receive_terrain_chunk_mesh_data,
    TerrainChunkSystemSet,
};

#[derive(Default, Debug)]
pub struct TerrainChunkPlugin;

impl Plugin for TerrainChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainChunkMapper>()
            .configure_sets(
                Update,
                (TerrainChunkSystemSet::UpdateLoader,)
                    .chain()
                    .in_set(TerrainSystemSet::UpdateChunk),
            )
            .add_plugins(TerrainChunkLoaderPlugin)
            .add_plugins(ExtractComponentPlugin::<TerrainChunkState>::default())
            .add_plugins(ExtractComponentPlugin::<TerrainChunkAddress>::default())
            .add_plugins(ExtractComponentPlugin::<TerrainChunkAabb>::default())
            .add_plugins(ExtractComponentPlugin::<TerrainChunkSeamLod>::default())
            .add_plugins(ExtractComponentPlugin::<TerrainChunkNeighborLodNodes>::default())
            .add_plugins(ExtractComponentPlugin::<TerrainChunkBorderVertices>::default())
            .add_plugins(ExtractResourcePlugin::<TerrainChunkMapper>::default())
            .add_systems(PreUpdate, receive_terrain_chunk_mesh_data)
            .add_systems(PreUpdate, update_terrain_chunk_state)
            .add_observer(trigger_chunk_unload_event)
            .add_observer(trigger_chunk_reload_event)
            .add_observer(trigger_chunk_load_event);
    }
}

pub fn update_terrain_chunk_state(mut query: Query<&mut TerrainChunkState>) {
    for mut state in query.iter_mut() {
        if *state != TerrainChunkState::DONE {
            *state = TerrainChunkState::DONE;
        }
    }
}
