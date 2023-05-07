use bevy::prelude::*;
use simdnoise::NoiseBuilder;

use self::{
    coords::{TerrainChunkCoord, TerrainGlobalCoord, TerrainVoxelCoord},
    terrain::TerrainData,
    visible::VisibleTerrainRange,
};

use super::{
    bundle::{TerrainBundle, TerrainChunkBundle, TerrainVoxelBundle},
    noise_config::{TerrainNoiseConfig, TerrainNoiseData},
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
                apply_system_buffers,
                update_terrain_noise,
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
    query: Query<(&Children, &TerrainNoiseData), With<TerrainChunk>>,
    mut children_query: Query<
        (&mut TerrainVoxelCoord, &mut TerrainGlobalCoord),
        With<TerrainVoxel>,
    >,
) {
    for (chilren, nosie_data) in query.iter() {
        for child in chilren.iter() {
            let (mut voxel_coord, mut global_coord) = children_query.get_mut(*child).unwrap();

            let height = nosie_data
                .noise
                .get(voxel_coord.x + voxel_coord.z * TERRAIN_VOXEL_NUM_IN_CHUNK)
                .unwrap();

            voxel_coord.y = *height as usize;
            global_coord.y = voxel_coord.y as i64;
        }
    }
}

fn update_terrain_noise(
    mut all_chunks: Query<(&mut TerrainNoiseData, &TerrainChunkCoord), With<TerrainChunk>>,
    noise_config: Res<TerrainNoiseConfig>,
) {
    for (mut noise_data, chunk_coord) in all_chunks.iter_mut() {
        if noise_data.noise.len() > 0 {
            continue;
        }

        let offset_x = (chunk_coord.x * TERRAIN_VOXEL_NUM_IN_CHUNK as i64) as f32;
        let offset_z = (chunk_coord.z * TERRAIN_VOXEL_NUM_IN_CHUNK as i64) as f32;

        let (noise, min, max) = NoiseBuilder::fbm_2d_offset(
            offset_x,
            TERRAIN_VOXEL_NUM_IN_CHUNK,
            offset_z,
            TERRAIN_VOXEL_NUM_IN_CHUNK,
        )
        .with_seed(noise_config.seed)
        .with_freq(noise_config.frequency)
        .with_gain(noise_config.gain)
        .with_lacunarity(noise_config.lacunarity)
        .with_octaves(noise_config.octaves)
        .generate();

        info!("nosie offset:{}:{} => {}:{}", offset_x, offset_z, min, max);

        noise_data
            .noise
            .resize(TERRAIN_VOXEL_NUM_IN_CHUNK * TERRAIN_VOXEL_NUM_IN_CHUNK, 0.0);
        let scaled_min = noise_config.y_range.start as f32;
        let scaled_max = noise_config.y_range.end as f32;
        for (i, value) in noise.iter().enumerate() {
            // let alpha = (*value - min) / (max - min);
            noise_data.noise[i] = scaled_min + (scaled_max - scaled_min) * *value;
        }

        // info!(
        //     "Terrain nosie value len: {}, offset [{}:{}:{}]",
        //     noise_data.noise.len(),
        //     offset_x,
        //     offset_y,
        //     offset_z
        // );
    }
}
