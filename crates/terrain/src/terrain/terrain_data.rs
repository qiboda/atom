use bevy::{prelude::*, utils::HashMap};

use crate::visible::{
    visible_areas::{TerrainSingleVisibleArea, TerrainVisibleAreas},
    visible_range::VisibleTerrainRange,
};

use super::{
    bundle::TerrainBundle,
    chunk::{TerrainChunk, TerrainChunkBundle},
    TerrainSystemSet,
};

use terrain_core::chunk::coords::TerrainChunkCoord;

#[derive(Debug, Component, Default)]
pub struct TerrainData {
    /// entity is TerrainChunk
    pub data: HashMap<TerrainChunkCoord, Entity>,
}

impl TerrainData {
    pub fn get_chunk_entity_by_coord(
        &self,
        terrain_chunk_coord: TerrainChunkCoord,
    ) -> Option<&Entity> {
        self.data.get(&terrain_chunk_coord)
    }

    pub fn new() -> TerrainData {
        Self::default()
    }
}

#[derive(Debug, Component)]
pub struct Terrain;

#[derive(Default, Debug)]
pub struct TerrainDataPlugin;

impl Plugin for TerrainDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_terrain)
            .add_systems(
                Update,
                create_visible_chunks.in_set(TerrainSystemSet::GenerateTerrain),
            )
            .add_systems(
                Last,
                despawn_entity.in_set(TerrainSystemSet::GenerateTerrain),
            );
    }
}

// #[bevycheck::system]
fn setup_terrain(mut commands: Commands) {
    commands.spawn(TerrainBundle::default()).insert(Terrain);
}

// #[bevycheck::system]
#[allow(clippy::type_complexity)]
fn create_visible_chunks(
    mut commands: Commands,
    terrain_areas: Res<TerrainVisibleAreas>,
    visible_changed_query: Query<
        Entity,
        (
            Or<(Changed<VisibleTerrainRange>, Changed<GlobalTransform>)>,
            With<VisibleTerrainRange>,
        ),
    >,
    mut terrain_query: Query<(Entity, &mut TerrainData), With<Terrain>>,
) {
    for entity in visible_changed_query.iter() {
        let last_terrain_single_visible_area = match terrain_areas.get_last_visible_area(entity) {
            Some(visible_area) => visible_area.clone(),
            None => TerrainSingleVisibleArea::default(),
        };

        let current_terrain_single_visible_area =
            match terrain_areas.get_current_visible_area(entity) {
                Some(visible_area) => visible_area.clone(),
                None => TerrainSingleVisibleArea::default(),
            };

        let mut add_count = 0;
        let (terrain_entity, mut terrain_data) = terrain_query.single_mut();
        current_terrain_single_visible_area.iter_chunk(&mut |x, y, z| {
            if last_terrain_single_visible_area.is_in_chunk(x, y, z) {
                return;
            }

            let chunk_coord = TerrainChunkCoord::from(&[x, y, z]);
            if terrain_data.data.contains_key(&chunk_coord) {
                return;
            }

            spawn_terrain_chunks(
                &mut commands,
                terrain_entity,
                chunk_coord,
                &mut terrain_data,
            );
            add_count += 1;
        });

        if add_count > 0 {
            info!("added count: {}", add_count);
        }
    }
}

fn spawn_terrain_chunks(
    commands: &mut Commands,
    terrain_entity: Entity,
    terrain_chunk_coord: TerrainChunkCoord,
    terrain_data: &mut TerrainData,
) {
    let child = commands
        .spawn((TerrainChunkBundle {
            terrain_chunk_coord,
            ..default()
        },))
        .insert(TerrainChunk)
        .id();

    info!("spawn_terrain_chunks: {:?}", terrain_chunk_coord);

    let mut terrian = commands.get_entity(terrain_entity).unwrap();
    terrian.add_child(child);
    terrain_data.data.insert(terrain_chunk_coord, child);
}

#[allow(clippy::type_complexity)]
fn despawn_entity(
    mut commands: Commands,
    terrain_areas: Res<TerrainVisibleAreas>,
    visible_changed_query: Query<
        Entity,
        (
            Or<(Changed<VisibleTerrainRange>, Changed<GlobalTransform>)>,
            With<VisibleTerrainRange>,
        ),
    >,
    mut terrain_query: Query<&mut TerrainData, With<Terrain>>,
) {
    for entity in visible_changed_query.iter() {
        let last_terrain_single_visible_area = match terrain_areas.get_last_visible_area(entity) {
            Some(visible_area) => visible_area.clone(),
            None => TerrainSingleVisibleArea::default(),
        };

        let current_terrain_single_visible_area =
            match terrain_areas.get_current_visible_area(entity) {
                Some(visible_area) => visible_area.clone(),
                None => TerrainSingleVisibleArea::default(),
            };

        let mut removed_count = 0;
        let mut terrain_data = terrain_query.single_mut();
        last_terrain_single_visible_area.iter_chunk(&mut |x, y, z| {
            if current_terrain_single_visible_area.is_in_chunk(x, y, z) {
                return;
            }

            if let Some(&terrain_chunk_entity) =
                terrain_data.get_chunk_entity_by_coord(TerrainChunkCoord::from(&[x, y, z]))
            {
                commands.entity(terrain_chunk_entity).despawn_recursive();
                terrain_data
                    .data
                    .remove(&TerrainChunkCoord::from(&[x, y, z]));
                removed_count += 1;
            }
        });

        if removed_count > 0 {
            info!("removed count: {}", removed_count);
        }
    }
}
