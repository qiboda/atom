use bevy::prelude::*;

use crate::TerrainSystemSet;

use super::{
    chunk_mapper::{trigger_lod_octree_add, trigger_lod_octree_remove, TerrainChunkMapper},
    event::{
        hidden_main_mesh, to_create_seam_mesh, update_create_seam_mesh_over,
        update_to_wait_create_seam,
    },
};

#[derive(Default, Debug)]
pub struct TerrainChunkPlugin;

impl Plugin for TerrainChunkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainChunkMapper>()
            .observe(trigger_lod_octree_add)
            .observe(trigger_lod_octree_remove)
            .add_systems(Last, hidden_main_mesh)
            .add_systems(
                Update,
                (
                    // 按照事件发送的顺序执行
                    update_to_wait_create_seam,
                    to_create_seam_mesh,
                    update_create_seam_mesh_over,
                )
                    .chain()
                    .in_set(TerrainSystemSet::UpdateChunk),
            );
    }
}
