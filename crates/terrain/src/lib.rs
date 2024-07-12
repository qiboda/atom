pub mod bundle;
pub mod chunk;
pub mod ecology;
pub mod isosurface;
pub mod materials;
pub mod setting;
pub mod visible;

use bevy::{pbr::ExtendedMaterial, prelude::*};
use bundle::TerrainBundle;
use chunk::chunk_mapper::TerrainChunkPlugin;
use isosurface::IsosurfaceExtractionPlugin;
use materials::{terrain::TerrainMaterial, terrain_debug::TerrainDebugMaterial};
use setting::{TerrainChunkSettings, TerrainClipMapSettings, TerrainSetting};
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
            chunk_settings: TerrainChunkSettings::default(),
            clipmap_settings: TerrainClipMapSettings::default(),
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
        .add_plugins(MaterialPlugin::<
            ExtendedMaterial<StandardMaterial, TerrainMaterial>,
        >::default())
        .add_plugins(MaterialPlugin::<TerrainDebugMaterial>::default())
        .add_plugins(TerrainVisibleAreaPlugin)
        .add_plugins(TerrainChunkPlugin)
        .add_plugins(IsosurfaceExtractionPlugin)
        .add_systems(Startup, setup_terrain);
    }
}

fn setup_terrain(mut commands: Commands) {
    commands.spawn(TerrainBundle::default());
}
