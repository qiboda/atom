/// TODO lod和chunk的删除和更改，使用弱边界。
/// TODO 销毁的Chunk，存储到文件中。
/// TODO 纹理数组的支持，还是使用standard material 还是自定义材质。
/// TODO 地形的用户修改。
/// TODO 如何将切断的mesh，施加重力。
/// TODO 如何支持地形的函数自定义，以及曲线修改地形。
/// TODO 自定义地形的函数组合。
/// TODO 水面的支持。
/// TODO 河流的支持。以及小路的生成。(小路或许可以靠寻路系统生成)
/// TODO 地形和生态的分布。
/// TODO 缓存密度函数的值，避免重复计算。
/// TODO 视窗剔除。
pub mod chunk_mgr;
pub mod isosurface;
pub mod lod;
pub mod setting;
pub mod utils;

use atom_internal::app_state::AppState;
use bevy::prelude::*;
use chunk_mgr::plugin::TerrainChunkPlugin;
use isosurface::IsosurfaceExtractionPlugin;
use lod::lod_octree::TerrainLodOctreePlugin;
use setting::TerrainSetting;
use settings::SettingPlugin;

#[derive(Debug, Default)]
pub struct TerrainSubsystemPlugin;

impl Plugin for TerrainSubsystemPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(SettingPlugin::<TerrainSetting>::default())
            .configure_sets(
                Update,
                (
                    TerrainSystemSet::UpdateLodOctree,
                    TerrainSystemSet::UpdateChunk,
                    TerrainSystemSet::GenerateTerrain,
                )
                    .chain()
                    .run_if(in_state(AppState::AppRunning)),
            )
            .add_plugins(TerrainLodOctreePlugin)
            .add_plugins(TerrainChunkPlugin)
            .add_plugins(IsosurfaceExtractionPlugin);
    }
}

#[derive(Debug, Reflect, SystemSet, PartialEq, Eq, Hash, Clone)]
pub enum TerrainSystemSet {
    UpdateLodOctree,
    UpdateChunk,
    GenerateTerrain,
}

#[derive(Component, Debug, Default)]
pub struct TerrainObserver;
