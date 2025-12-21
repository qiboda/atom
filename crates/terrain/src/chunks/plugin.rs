use bevy::{prelude::*, render::extract_component::ExtractComponentPlugin};

use crate::chunks::chunk::TerrainChunkCoord;
use crate::chunks::loader;

#[derive(Default, Debug)]
pub struct TerrainChunkPlugin;

impl Plugin for TerrainChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractComponentPlugin::<TerrainChunkCoord>::default());

        // 添加 chunk loader 系统
        loader::add_chunk_loader_systems(app);
    }
}
