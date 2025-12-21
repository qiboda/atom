pub mod setting;

use bevy::{prelude::*, render::extract_resource::ExtractResourcePlugin};
use setting::TerrainSetting;

use crate::chunks::plugin::TerrainChunkPlugin;

#[derive(Debug, Hash, PartialEq, Eq, Clone, States, Default)]
pub enum TerrainState {
    #[default]
    None,
    // 加载资源，如地形纹理等资产。
    LoadAssets,
    // 生成地形的大致基础信息
    GenerateTerrainSketch,
    // 生成地形的高度图。
    GenerateHeightMap,
    // 生成地形的Mesh
    GenerateTerrainMesh,
}

#[derive(Debug, Reflect, SystemSet, PartialEq, Eq, Hash, Clone)]
pub enum TerrainSystems {
    ChunkLoader,
    ApplyCSG,
    GenerateChunk,
}

#[derive(Debug, Default)]
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainSetting::default())
            .add_plugins(ExtractResourcePlugin::<TerrainSetting>::default());

        app.init_state::<TerrainState>();

        app.configure_sets(
            Update,
            (
                TerrainSystems::ChunkLoader,
                TerrainSystems::ApplyCSG,
                TerrainSystems::GenerateChunk,
            )
                .chain(), // .run_if(in_state(TerrainState::GenerateTerrainMesh)),
        );

        app.add_plugins(TerrainChunkPlugin);
    }
}
