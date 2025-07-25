use std::{collections::HashMap, sync::Arc};

use bevy::{
    app::{Plugin, Update},
    asset::Assets,
    log::warn,
    math::{UVec2, Vec3},
    prelude::{
        App, Commands, Component, Deref, DerefMut, Entity, Event, EventReader, EventWriter,
        IntoSystemConfigs, Query, Res, ResMut, Resource, With,
    },
};
use bevy_landmass::{
    ArchipelagoRef3d, Island, LandmassSystemSet, NavMeshHandle, NavMeshHandle3d, NavigationMesh3d,
};
// use oxidized_navigation::tiles::NavMeshTile;

/// The main plugin that updates the `landmass` archipelago using
/// `oxidized_navigation` navigation meshes.
pub struct LandmassOxidizedNavigationPlugin;

impl Plugin for LandmassOxidizedNavigationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastTileGenerations>()
            .add_event::<UpdateTile>()
            .init_resource::<TileToIsland>()
            .add_systems(
                Update,
                (LastTileGenerations::system, UpdateTile::system)
                    .chain()
                    .before(LandmassSystemSet::SyncExistence),
            );
    }
}

/// A marker component to mark the [`bevy_landmass::Archipelago3d`] to put
/// `oxidized_navigation` navigation meshes into. Note exactly one entity must
/// have this marker.
#[derive(Component)]
pub struct OxidizedArchipelago;

/// A resource tracking the previous tile generations. This works around the
/// lack of https://github.com/TheGrimsey/oxidized_navigation/issues/31.
#[derive(Resource, Default, Deref, DerefMut)]
struct LastTileGenerations(HashMap<UVec2, u64>);

impl LastTileGenerations {
    /// The system updating these tile generations and sending [`UpdateTile`]
    /// events in response to changes.
    fn system(
        oxidized_nav_mesh: Res<oxidized_navigation::NavMesh>,
        mut last_tile_generations: ResMut<Self>,
        mut update_events: EventWriter<UpdateTile>,
    ) {
        let oxidized_nav_mesh = oxidized_nav_mesh.get();
        let tiles = match oxidized_nav_mesh.read() {
            Ok(tiles) => tiles,
            Err(err) => {
                warn!("Failed to read oxidized_navigation::NavMesh: {err}");
                return;
            }
        };

        for (&tile, &generation) in tiles.tile_generations.iter() {
            let last_generation = last_tile_generations
                .entry(tile)
                .and_modify(|last_generation| {
                    if *last_generation == generation {
                        return;
                    }
                    update_events.send(UpdateTile(tile));
                })
                .or_insert_with(|| {
                    update_events.send(UpdateTile(tile));
                    generation
                });
            *last_generation = generation;
        }
    }
}

/// An event to update a specific tile of `oxidized_navigation`. This works
/// around the lack of
/// https://github.com/TheGrimsey/oxidized_navigation/issues/31.
#[derive(Event)]
struct UpdateTile(UVec2);

impl UpdateTile {
    /// Updates the island (or creates one) corresponding to the tile in the
    /// event.
    fn system(
        mut events: EventReader<Self>,
        oxidized_nav_mesh: Res<oxidized_navigation::NavMesh>,
        archipelago: Query<Entity, With<OxidizedArchipelago>>,
        mut nav_meshes: ResMut<Assets<bevy_landmass::NavMesh3d>>,
        mut tile_to_entity: ResMut<TileToIsland>,
        mut commands: Commands,
    ) {
        let oxidized_nav_mesh = oxidized_nav_mesh.get();
        let tiles = match oxidized_nav_mesh.read() {
            Ok(tiles) => tiles,
            Err(err) => {
                warn!("Failed to read oxidized_navigation::NavMesh: {err}");
                return;
            }
        };

        for Self(tile) in events.read() {
            let entity = tile_to_entity.get(tile);
            let nav_mesh_tile = tiles.tiles.get(tile);

            let nav_mesh_tile = match nav_mesh_tile {
                None => {
                    if let Some(&entity) = entity {
                        // Ensure the island entity has no nav mesh on it. This may be
                        // redundant if the generation is spuriously incremented, however
                        // that should be infrequent so that should be fine.
                        commands.entity(entity).remove::<NavMeshHandle3d>();
                    }
                    continue;
                }
                Some(tile) => tile,
            };

            let entity = match entity {
                None => {
                    let archipelago = archipelago.single();
                    let entity = commands
                        .spawn((Island, ArchipelagoRef3d::new(archipelago)))
                        .id();
                    tile_to_entity.0.insert(*tile, entity);
                    entity
                }
                Some(&entity) => entity,
            };

            let nav_mesh = tile_to_landmass_nav_mesh(nav_mesh_tile);
            let valid_nav_mesh = match nav_mesh.validate() {
                Ok(valid_nav_mesh) => valid_nav_mesh,
                Err(err) => {
                    warn!("Failed to validate oxidized_navigation tile: {err:?}");
                    // Ensure the island has no nav mesh. The island may be brand new, so
                    // this may be redundant, but better to make sure.
                    commands.entity(entity).remove::<NavMeshHandle3d>();
                    continue;
                }
            };

            commands.entity(entity).insert(NavMeshHandle(nav_meshes.add(
                bevy_landmass::NavMesh3d {
                    nav_mesh: Arc::new(valid_nav_mesh),
                    type_index_to_node_type: HashMap::new(),
                },
            )));
        }
    }
}

/// Converts the [`NavMeshTile`] into the corresponding [`NavigationMesh3d`].
fn tile_to_landmass_nav_mesh(tile: &NavMeshTile) -> NavigationMesh3d {
    NavigationMesh3d {
        vertices: tile
            .vertices
            .iter()
            .copied()
            .map(|vertex| Vec3::new(vertex.x, vertex.y, vertex.z))
            .collect(),
        polygons: tile
            .polygons
            .iter()
            .map(|polygon| {
                polygon
                    .indices
                    .iter()
                    .copied()
                    .map(|i| i as usize)
                    .collect()
            })
            .collect(),
        polygon_type_indices: vec![0; tile.polygons.len()],
    }
}

/// Tracks the islands corresponding to each tile of `oxidized_navigation`.
#[derive(Resource, Default, Deref)]
pub struct TileToIsland(HashMap<UVec2, Entity>);
