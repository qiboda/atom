use bevy::{prelude::*, utils::HashMap};

use crate::{
    chunk_mgr::chunk::{bundle::TerrainChunkBundle, state::TerrainChunkState},
    isosurface::comp::TerrainChunkUpdateLodEvent,
    setting::TerrainSetting,
    visible::{visible_areas::TerrainSingleVisibleAreaProxy, visible_range::VisibleTerrainRange},
    TerrainSystemSet,
};

use terrain_core::chunk::coords::TerrainChunkCoord;

use super::{
    bundle::Terrain,
    chunk::{bundle::TerrainChunk, chunk_lod::TerrainChunkLod},
    event::read_main_mesh_created_event,
};

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
            (
                read_main_mesh_created_event,
                update_visible_chunks,
                update_visible_chunks_lod,
            )
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
    let mut bundle = TerrainChunkBundle::new(TerrainChunkState::Done);
    bundle.chunk_coord = terrain_chunk_coord;
    let child = commands.spawn(bundle).id();

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
    mut terrain_chunk_query: Query<
        (
            Entity,
            &TerrainChunkCoord,
            &mut TerrainChunkLod,
            &mut TerrainChunkState,
        ),
        With<TerrainChunk>,
    >,
    mut event_writer: EventWriter<TerrainChunkUpdateLodEvent>,
) {
    for visible_area in visible_changed_query.iter() {
        let current_visible_area = visible_area.get_current();

        let terrain_data = terrain_query.single();
        current_visible_area.iter_chunk(&mut |x, y, z| {
            let chunk_coord = TerrainChunkCoord::from(&[x, y, z]);
            if let Some(chunk_entity) = terrain_data.get_chunk_entity_by_coord(chunk_coord) {
                if let Ok((chunk_entity, terrain_chunk_coord, mut chunk_lod, mut chunk_state)) =
                    terrain_chunk_query.get_mut(*chunk_entity)
                {
                    assert_eq!(terrain_chunk_coord, &chunk_coord);
                    let chunk_coord_diff = &current_visible_area.center_chunk_coord - &chunk_coord;
                    if let Some(clipmap_lod) =
                        terrain_settings.clipmap_settings.get_lod(chunk_coord_diff)
                    {
                        chunk_lod.set_lod(clipmap_lod.lod);
                        // 最新创建的Chunk，lod总是dirty。也会触发这个事件。
                        if chunk_lod.is_dirty() {
                            event_writer.send(TerrainChunkUpdateLodEvent {
                                entity: chunk_entity,
                                lod: chunk_lod.get_lod(),
                            });
                            *chunk_state = TerrainChunkState::CreateMainMesh(chunk_lod.get_lod());
                            chunk_lod.clean_dirty();
                            info!(
                                "terrain_chunk_coord: {:?}, lod: {}, final {}",
                                terrain_chunk_coord,
                                clipmap_lod.lod,
                                chunk_lod.get_lod()
                            );
                        }
                    } else {
                        error!("{:?} is not a valid lod distance", chunk_coord_diff);
                    }
                }
            }
        });
    }
}
