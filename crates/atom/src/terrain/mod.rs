/// create chunk
/// get iso iso_surface to chunk
/// chunk eval iso_surface and get value
/// apply patch to chunk
/// generate cubes, use octree store since easy to search.
///
pub mod bundle;
pub mod chunk;
pub mod isosurface;
pub mod settings;
pub mod terrain;
pub mod visible;

use bevy::prelude::*;

use self::{
    chunk::TerrainDataPlugin,
    visible::visible_areas::{TerrainVisibleAreaPlugin, TerrainVisibleAreas},
};

#[derive(SystemSet, PartialEq, Eq, Debug, Clone, Hash)]
enum TerrainSystemSet {
    VisibleAreas,
    GenerateTerrain,
}

#[derive(Default, Debug)]
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainVisibleAreas::default())
            .configure_sets(
                Update,
                (
                    TerrainSystemSet::VisibleAreas,
                    TerrainSystemSet::GenerateTerrain,
                )
                    .chain(),
            )
            .add_plugins(TerrainVisibleAreaPlugin)
            .add_plugins(TerrainDataPlugin);
    }
}
