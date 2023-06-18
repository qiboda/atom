use bevy::prelude::*;

use self::{
    coords::{TerrainChunkCoord, TerrainGlobalCoord, TerrainVoxelCoord, VoxelGradedCoord},
    terrain::TerrainData,
    visible::VisibleTerrainRange,
};

use super::{
    bundle::{TerrainBundle, TerrainChunkBundle, TerrainVoxelBundle},
    iso_surface::surface::TerrainSurfaceData,
    visible_areas::{TerrainSingleVisibleArea, TerrainVisibleAreas},
    TerrainSystemSet,
};

pub mod chunk;
pub mod coords;
pub mod terrain;
pub mod visible;
pub mod voxel;

// voxel total size in one chunk
pub const TERRAIN_VOXEL_NUM_IN_CHUNK: usize = 16;

// voxel size in meter unit
pub const TERRAIN_VOXEL_SIZE: f32 = 1.0;

#[derive(Debug, Component)]
pub struct Terrain;

#[derive(Debug, Component)]
pub struct TerrainChunk;

#[derive(Debug, Component)]
pub struct TerrainVoxel;

#[derive(Default, Debug)]
pub struct TerrainDataPlugin;

impl Plugin for TerrainDataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_terrain).add_systems(
            Update,
            (
                create_visible_chunks,
                apply_deferred,
                update_terrain_surface,
                update_terrain_voxel_height,
            )
                .chain()
                .in_set(TerrainSystemSet::TerrainData),
        );
    }
}

// #[bevycheck::system]
fn setup_terrain(mut commands: Commands) {
    commands.spawn(TerrainBundle::default()).insert(Terrain);
}

// #[bevycheck::system]
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
                removed_count = removed_count + 1;
            }
        });

        if add_count > 0 {
            info!("added count: {}", add_count);
        }
        if removed_count > 0 {
            info!("removed count: {}", removed_count);
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
        .spawn(TerrainChunkBundle {
            terrain_chunk_coord,
            ..default()
        })
        .insert(TerrainChunk)
        .with_children(|parent| {
            for x in 0..TERRAIN_VOXEL_NUM_IN_CHUNK {
                for z in 0..TERRAIN_VOXEL_NUM_IN_CHUNK {
                    parent
                        .spawn(TerrainVoxelBundle {
                            local_coord: TerrainVoxelCoord::from(&[x, 0, z]),
                            global_coord: TerrainGlobalCoord::from_local_coords(
                                &terrain_chunk_coord,
                                &TerrainVoxelCoord::from(&[x, 0, z]),
                            ),
                            ..default()
                        })
                        .insert(TerrainVoxel);
                }
            }
        })
        .id();

    let mut terrian = commands.get_entity(terrain_entity).unwrap();
    terrian.add_child(child);
    terrain_data.data.insert(terrain_chunk_coord, child);
}

fn update_terrain_voxel_height(
    query: Query<&Children, With<TerrainChunk>>,
    mut children_query: Query<
        (&mut TerrainVoxelCoord, &mut TerrainGlobalCoord),
        With<TerrainVoxel>,
    >,
    surface_data: Res<TerrainSurfaceData>,
) {
    for children in query.iter() {
        for child in children.iter() {
            let (mut voxel_coord, mut global_coord) = children_query.get_mut(*child).unwrap();

            let height = surface_data.get_surface_value(&VoxelGradedCoord::from(&*global_coord));

            voxel_coord.y = height as usize;
            global_coord.y = voxel_coord.y as i64;
        }
    }
}

fn update_terrain_surface(
    mut all_chunks: Query<&TerrainChunkCoord, With<TerrainChunk>>,
    surface_data: ResMut<TerrainSurfaceData>,
) {
    for chunk_coord in all_chunks.iter_mut() {
        surface_data.generate_surface_value(chunk_coord);
    }
}
