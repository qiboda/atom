/// create chunk
/// get iso iso_surface to chunk
/// chunk eval iso_surface and get value
/// apply patch to chunk
/// generate cubes, use octree store since easy to search.
///
pub mod bundle;
pub mod cube;
pub mod data;
pub mod iso_surface;
pub mod visible_areas;

use bevy::prelude::*;

use self::{
    cube::TerrainCubePlugin,
    data::TerrainDataPlugin,
    iso_surface::IsoSurfacePlugin,
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
            .add_plugin(IsoSurfacePlugin)
            .add_plugin(TerrainDataPlugin)
            .add_plugin(TerrainCubePlugin);
    }
}
