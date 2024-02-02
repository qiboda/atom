/// create chunk
/// get iso iso_surface to chunk
/// chunk eval iso_surface and get value
/// apply patch to chunk
/// generate cubes, use octree store since easy to search.
///
pub mod bundle;
pub mod chunk;
pub mod ecology;
pub mod isosurface;
pub mod materials;
pub mod settings;
pub mod terrain_data;

use bevy::{pbr::ExtendedMaterial, prelude::*};

use crate::visible::TerrainVisibleAreaPlugin;

use self::{
    isosurface::IsosurfaceExtractionPlugin, materials::terrain::TerrainMaterial,
    terrain_data::TerrainDataPlugin,
};

#[derive(SystemSet, PartialEq, Eq, Debug, Clone, Hash)]
pub enum TerrainSystemSet {
    VisibleAreas,
    GenerateTerrain,
}

#[derive(Default, Debug)]
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                TerrainSystemSet::VisibleAreas,
                TerrainSystemSet::GenerateTerrain,
            )
                .chain(),
        )
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, TerrainMaterial>,
        >::default())
        .add_plugins(TerrainVisibleAreaPlugin)
        .add_plugins(TerrainDataPlugin)
        .add_plugins(IsosurfaceExtractionPlugin);
    }
}
