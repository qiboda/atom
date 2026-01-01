pub mod chunk;
pub mod loader;
pub mod mesh;

use bevy::{prelude::*, render::extract_component::ExtractComponentPlugin};

use crate::chunks::chunk::{TerrainChunk, TerrainChunkCoord};
use crate::chunks::loader::TerrainChunkLoaderPlugin;
use crate::chunks::mesh::TerrainChunkMeshingPlugin;

#[derive(Default, Debug)]
pub struct TerrainChunkPlugin;

impl Plugin for TerrainChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<TerrainChunk>::default());
        app.add_plugins(ExtractComponentPlugin::<TerrainChunkCoord>::default());

        app.add_plugins(TerrainChunkLoaderPlugin);
        app.add_plugins(TerrainChunkMeshingPlugin);
    }
}
