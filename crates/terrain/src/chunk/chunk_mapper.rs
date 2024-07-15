use bevy::{prelude::*, utils::HashMap};

use crate::{
    bundle::Terrain,
    chunk::{chunk_data::TerrainChunkData, TerrainChunk, TerrainChunkBundle},
    setting::TerrainSetting,
    visible::{visible_areas::TerrainSingleVisibleAreaProxy, visible_range::VisibleTerrainRange},
    TerrainSystemSet,
};

use terrain_core::chunk::coords::TerrainChunkCoord;

#[derive(Debug, Component, Default, Reflect)]
pub struct TerrainChunkMapper {
    /// entity is TerrainChunk
    pub data: HashMap<TerrainChunkCoord, Entity>,
}

impl TerrainChunkMapper {
    pub fn get_chunk_entity_by_coord(
        &self,
        terrain_chunk_coord: TerrainChunkCoord,
    ) -> Option<&Entity> {
        self.data.get(&terrain_chunk_coord)
    }

    pub fn new() -> TerrainChunkMapper {
        Self::default()
    }
}

#[derive(Default, Debug)]
pub struct TerrainChunkPlugin;

impl Plugin for TerrainChunkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_visible_chunks, update_visible_chunks_lod)
                .chain()
                .in_set(TerrainSystemSet::UpdateChunk),
        );
    }
}

#[allow(clippy::type_complexity)]
fn update_visible_chunks(
    mut commands: Commands,
    visible_changed_query: Query<
        &TerrainSingleVisibleAreaProxy,
        (
            Changed<TerrainSingleVisibleAreaProxy>,
            With<VisibleTerrainRange>,
        ),
    >,
    mut terrain_query: Query<(Entity, &mut TerrainChunkMapper), With<Terrain>>,
) {
    for visible_area in visible_changed_query.iter() {
        let last_terrain_single_visible_area = visible_area.get_last();
        let current_terrain_single_visible_area = visible_area.get_current();

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
            debug!("added count: {}", add_count);
        }

        let mut removed_count = 0;
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
            debug!("removed count: {}", removed_count);
        }
    }
}

fn spawn_terrain_chunks(
    commands: &mut Commands,
    terrain_entity: Entity,
    terrain_chunk_coord: TerrainChunkCoord,
    terrain_data: &mut TerrainChunkMapper,
) {
    let child = commands
        .spawn((TerrainChunkBundle {
            terrain_chunk_coord,
            ..default()
        },))
        .id();

    debug!("spawn_terrain_chunks: {:?}", terrain_chunk_coord);

    let mut terrian = commands.get_entity(terrain_entity).unwrap();
    terrian.add_child(child);
    terrain_data.data.insert(terrain_chunk_coord, child);
}

// 遍历每个可见区域，更新可见区域的LOD
fn update_visible_chunks_lod(
    terrain_settings: Res<TerrainSetting>,
    visible_changed_query: Query<
        &TerrainSingleVisibleAreaProxy,
        (
            Changed<TerrainSingleVisibleAreaProxy>,
            With<VisibleTerrainRange>,
        ),
    >,
    terrain_query: Query<&TerrainChunkMapper, With<Terrain>>,
    mut terrain_chunk_query: Query<(&TerrainChunkCoord, &mut TerrainChunkData), With<TerrainChunk>>,
) {
    for visible_area in visible_changed_query.iter() {
        let current_terrain_single_visible_area = visible_area.get_current();

        let terrain_data = terrain_query.single();
        current_terrain_single_visible_area.iter_chunk(&mut |x, y, z| {
            let chunk_coord = TerrainChunkCoord::from(&[x, y, z]);
            if let Some(chunk_entity) = terrain_data.get_chunk_entity_by_coord(chunk_coord) {
                if let Ok((terrain_chunk_coord, mut chunk_data)) =
                    terrain_chunk_query.get_mut(*chunk_entity)
                {
                    assert_eq!(terrain_chunk_coord, &chunk_coord);
                    chunk_data.lod = 0;
                }
            }
        });
    }

    for visible_area in visible_changed_query.iter() {
        let current_terrain_single_visible_area = visible_area.get_current();

        let terrain_data = terrain_query.single();
        current_terrain_single_visible_area.iter_chunk(&mut |x, y, z| {
            let chunk_coord = TerrainChunkCoord::from(&[x, y, z]);
            if let Some(chunk_entity) = terrain_data.get_chunk_entity_by_coord(chunk_coord) {
                if let Ok((terrain_chunk_coord, mut chunk_data)) =
                    terrain_chunk_query.get_mut(*chunk_entity)
                {
                    assert_eq!(terrain_chunk_coord, &chunk_coord);
                    let chunk_coord_diff =
                        &current_terrain_single_visible_area.center_chunk_coord - &chunk_coord;
                    if let Some(clipmap_lod) =
                        terrain_settings.clipmap_settings.get_lod(chunk_coord_diff)
                    {
                        chunk_data.lod = chunk_data.lod.max(clipmap_lod.lod);
                        trace!("lod: {} final {}", clipmap_lod.lod, chunk_data.lod);
                    } else {
                        error!("{:?} is not a valid lod distance", chunk_coord_diff);
                    }
                }
            }
        });
    }
}
