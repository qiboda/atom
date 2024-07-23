use bevy::prelude::*;

use crate::TerrainSystemSet;

use super::{
    chunk_loader::TerrainChunkLoaderPlugin,
    chunk_mapper::{read_chunk_load_event, read_chunk_unload_event, TerrainChunkMapper},
    event::{
        hidden_main_mesh, to_create_seam_mesh, update_create_seam_mesh_over,
        update_to_wait_create_seam,
    },
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
            .add_systems(Last, hidden_main_mesh)
            .add_systems(
                Update,
                (
                    // 按照事件发送的顺序执行
                    read_chunk_unload_event,
                    read_chunk_load_event,
                    update_to_wait_create_seam,
                    to_create_seam_mesh,
                    update_create_seam_mesh_over,
                )
                    .chain()
                    .in_set(TerrainChunkSystemSet::UpdateChunk),
            );
    }
}
