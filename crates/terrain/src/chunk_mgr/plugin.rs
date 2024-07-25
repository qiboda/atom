use bevy::prelude::*;

use crate::TerrainSystemSet;

use super::{
    chunk_loader::TerrainChunkLoaderPlugin,
    chunk_mapper::{
        trigger_chunk_load_event, trigger_chunk_reload_event, trigger_chunk_unload_event,
        TerrainChunkMapper,
    },
    event::{to_create_seam_mesh, update_create_seam_mesh_over, update_to_wait_create_seam},
    TerrainChunkSystemSet,
};

#[derive(Default, Debug)]
pub struct TerrainChunkPlugin;

impl Plugin for TerrainChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainChunkMapper>()
            .configure_sets(
                Update,
                (
                    TerrainChunkSystemSet::UpdateLoader,
                    TerrainChunkSystemSet::UpdateChunk,
                )
                    .chain()
                    .in_set(TerrainSystemSet::UpdateChunk),
            )
            .add_plugins(TerrainChunkLoaderPlugin)
            .observe(trigger_chunk_unload_event)
            .observe(trigger_chunk_reload_event)
            .observe(trigger_chunk_load_event)
            .add_systems(
                Update,
                (
                    update_to_wait_create_seam,
                    to_create_seam_mesh,
                    update_create_seam_mesh_over,
                )
                    .chain()
                    .in_set(TerrainChunkSystemSet::UpdateChunk),
            );
    }
}
