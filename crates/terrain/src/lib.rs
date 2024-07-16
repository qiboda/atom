/// TODO: lod和chunk的删除和更改，使用弱边界。
/// TODO: 等待setting加载完毕了再生成地形。
/// TODO: 销毁的Chunk，存储到文件中。
pub mod chunk_mgr;
pub mod isosurface;
pub mod setting;
pub mod visible;

use atom_internal::app_state::AppState;
use bevy::prelude::*;
use chunk_mgr::plugin::TerrainChunkPlugin;
use isosurface::IsosurfaceExtractionPlugin;
use setting::TerrainSetting;
use settings::SettingPlugin;
use visible::TerrainVisibleAreaPlugin;

#[derive(Debug, Default)]
pub struct TerrainSubsystemPlugin;

impl Plugin for TerrainSubsystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SettingPlugin::<TerrainSetting>::default())
            .configure_sets(
                Update,
                (
                    TerrainSystemSet::VisibleAreas,
                    TerrainSystemSet::UpdateChunk,
                    TerrainSystemSet::GenerateTerrain,
                )
                    .chain()
                    .run_if(in_state(AppState::AppRunning)),
            )
            .add_plugins(TerrainVisibleAreaPlugin)
            .add_plugins(TerrainChunkPlugin)
            .add_plugins(IsosurfaceExtractionPlugin);
    }
}

#[derive(Debug, Reflect, SystemSet, PartialEq, Eq, Hash, Clone)]
pub enum TerrainSystemSet {
    VisibleAreas,
    UpdateChunk,
    GenerateTerrain,
}
