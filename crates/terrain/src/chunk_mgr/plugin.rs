use bevy::prelude::*;

use crate::TerrainSystemSet;

use super::{
    chunk_mapper::{
        add_visible_chunks, remove_visible_chunks, update_visible_chunks_lod, TerrainChunkMapper,
    },
    event::{
        update_to_wait_create_seam, update_create_seam_mesh_over, remove_unused_seam_mesh,
        to_create_seam_mesh,
    },
};

#[derive(Default, Debug)]
pub struct TerrainChunkPlugin;

impl Plugin for TerrainChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainChunkMapper>()
            .add_systems(
                Update,
                (
                    // 按照事件发送的顺序执行
                    update_visible_chunks_lod,
                    update_to_wait_create_seam,
                    to_create_seam_mesh,
                    update_create_seam_mesh_over,
                )
                    .chain()
                    .in_set(TerrainSystemSet::UpdateChunk),
            )
            .add_systems(
                Last,
                (
                    add_visible_chunks,
                    remove_visible_chunks,
                    remove_unused_seam_mesh,
                ),
            );
    }
}
