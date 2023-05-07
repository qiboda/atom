pub mod bundle;
pub mod cube;
pub mod data;
pub mod noise_config;
pub mod visible_areas;

use bevy::prelude::*;

use self::{
    cube::TerrainCubePlugin,
    data::TerrainDataPlugin,
    noise_config::TerrainNoiseConfig,
    visible_areas::{TerrainVisibleAreaPlugin, TerrainVisibleAreas},
};

#[derive(SystemSet, PartialEq, Eq, Debug, Clone, Hash)]
enum TerrainSystemSet {
    VisibleAreas,
    TerrainData,
    TerrainCube,
}

#[derive(Default, Debug)]
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainVisibleAreas::default())
            .insert_resource(TerrainNoiseConfig {
                seed: 32,
                y_range: -16..16,
                frequency: 0.01,
                lacunarity: 2.0,
                gain: 0.5,
                octaves: 0,
            })
            .configure_sets(
                Update,
                (
                    TerrainSystemSet::VisibleAreas,
                    TerrainSystemSet::TerrainData,
                    TerrainSystemSet::TerrainCube,
                )
                    .chain(),
            )
            .add_plugin(TerrainVisibleAreaPlugin)
            .add_plugin(TerrainDataPlugin)
            .add_plugin(TerrainCubePlugin);
    }
}
