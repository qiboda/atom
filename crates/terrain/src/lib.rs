/// TODO: lod和chunk的删除和更改，使用弱边界。
/// TODO: Tree移动到子实体。
/// TODO: 等待setting加载完毕了再生成地形。
/// TODO: 销毁的Chunk，存储到文件中。
pub mod chunk_mgr;
pub mod isosurface;
pub mod setting;
pub mod visible;

use bevy::prelude::*;
use chunk_mgr::bundle::TerrainBundle;
use chunk_mgr::chunk_mapper::TerrainChunkPlugin;
use isosurface::IsosurfaceExtractionPlugin;
use setting::{TerrainChunkSetting, TerrainClipMapSetting, TerrainSetting};
use settings::SettingPlugin;
use visible::TerrainVisibleAreaPlugin;

#[derive(Debug, Reflect, SystemSet, PartialEq, Eq, Hash, Clone)]
pub enum TerrainSystemSet {
    VisibleAreas,
    UpdateChunk,
    GenerateTerrain,
}

#[derive(Debug, Default)]
pub struct TerrainSubsystemPlugin;

impl Plugin for TerrainSubsystemPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainSetting {
            chunk_settings: TerrainChunkSetting::default(),
            clipmap_settings: TerrainClipMapSetting::default(),
        })
        .add_plugins((SettingPlugin::<TerrainSetting> {
            paths: Default::default(),
        },))
        .configure_sets(
            Update,
            (
                TerrainSystemSet::VisibleAreas,
                TerrainSystemSet::UpdateChunk,
                TerrainSystemSet::GenerateTerrain,
            )
                .chain(),
        )
        .add_plugins(TerrainVisibleAreaPlugin)
        .add_plugins(TerrainChunkPlugin)
        .add_plugins(IsosurfaceExtractionPlugin)
        .add_systems(Startup, setup_terrain);
    }
}

fn setup_terrain(mut commands: Commands) {
    commands.spawn(TerrainBundle::default());
}
