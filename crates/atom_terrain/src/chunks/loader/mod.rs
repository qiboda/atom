pub mod loaded_chunks;
pub mod observer;
mod systems;

use bevy::prelude::*;

pub use loaded_chunks::{TerrainChunkLoadMsg, TerrainChunkUnloadMsg, TerrainLoadedChunks};
pub use systems::update_chunk_lod;
pub use systems::update_grid_chunks;

use crate::terrain::TerrainSystems;

#[derive(Default)]
pub struct TerrainChunkLoaderPlugin;

impl Plugin for TerrainChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainLoadedChunks>()
            .add_message::<TerrainChunkLoadMsg>()
            .add_message::<TerrainChunkUnloadMsg>()
            .add_systems(
                Update,
                (
                    update_grid_chunks.in_set(TerrainSystems::ChunkLoader),
                    // LOD 已禁用 — 俯视角固定最大精度
                    // update_chunk_lod.in_set(TerrainSystems::ChunkLoader),
                ),
            );
    }
}
