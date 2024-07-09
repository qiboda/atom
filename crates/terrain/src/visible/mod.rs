use bevy::prelude::*;

use crate::TerrainSystemSet;

use self::visible_areas::{
    add_terrain_visible_areas, remove_terrain_visible_areas, update_terrain_visible_areas,
};

pub mod visible_areas;
pub mod visible_range;

#[derive(Default, Debug)]
pub struct TerrainVisibleAreaPlugin;

impl Plugin for TerrainVisibleAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                add_terrain_visible_areas,
                update_terrain_visible_areas,
                remove_terrain_visible_areas,
            )
                .chain()
                .in_set(TerrainSystemSet::VisibleAreas),
        );
    }
}
