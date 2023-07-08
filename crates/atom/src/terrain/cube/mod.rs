pub mod terrain_cube_voxel_map;

use bevy::prelude::*;

use self::terrain_cube_voxel_map::{TerrainCubeData, TerrainVoxelCubeMap};

use super::{
    data::{chunk::TerrainChunkData, coords::TerrainGlobalCoord, TerrainChunk, TerrainVoxel},
    visible_areas::TerrainVisibleAreas,
    TerrainSystemSet,
};

#[derive(Default, Debug)]
pub struct TerrainCubePlugin;

impl Plugin for TerrainCubePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TerrainVoxelCubeMap::default())
            .insert_resource(TerrainCubeData::default())
            .add_systems(Startup, prepare_cube_data)
            .add_systems(
                Update,
                (
                    remove_terrain_cube,
                    create_terrain_cube,
                    // log_terrain_voxel_cube_count,
                    // update_visible_chunks,
                    // update_visible_sections,
                )
                    .chain()
                    .in_set(TerrainSystemSet::TerrainCube),
            );
    }
}

#[derive(Debug, Component, Default)]
pub struct TerrainCube;

// #[bevycheck::system]
fn prepare_cube_data(
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut terrain_cube_data: ResMut<TerrainCubeData>,
) {
    let mesh = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let material = materials.add(StandardMaterial { ..default() });

    terrain_cube_data.mesh = Some(mesh);
    terrain_cube_data.material = Some(material);
}

fn remove_terrain_cube(
    mut commands: Commands,
    mut terrain_cube_map: ResMut<TerrainVoxelCubeMap>,
    mut removed_voxel: RemovedComponents<TerrainVoxel>,
) {
    removed_voxel.iter().for_each(|removed_voxel_entity| {
        if let Some(cube_entity) = terrain_cube_map.get_voxel_cube(removed_voxel_entity) {
            commands.entity(*cube_entity).despawn_recursive();

            terrain_cube_map.remove_voxel_cube(removed_voxel_entity);
        }
    });
}

fn create_terrain_cube(
    mut commands: Commands,
    mut chunk: Query<(&Children, &mut TerrainChunkData), With<TerrainChunk>>,
    all_voxel_coords: Query<&TerrainGlobalCoord, With<TerrainVoxel>>,
    terrain_cube_data: Res<TerrainCubeData>,
    mut terrain_cube_map: ResMut<TerrainVoxelCubeMap>,
) {
    for (children, mut chunk_data) in chunk.iter_mut() {
        if chunk_data.loaded {
            continue;
        }

        chunk_data.loaded = true;

        for &child in children.iter() {
            let global_coord = all_voxel_coords.get(child).unwrap();
            let id = commands
                .spawn(PbrBundle {
                    mesh: terrain_cube_data.mesh.clone().unwrap(),
                    material: terrain_cube_data.material.clone().unwrap(),
                    transform: Transform::from_translation(global_coord.to_location()),
                    ..Default::default()
                })
                .insert(TerrainCube)
                .id();
            terrain_cube_map.set_voxel_cube(child, id);
        }
    }
}

fn _log_terrain_voxel_cube_count(
    cubes: Query<(), With<TerrainCube>>,
    voxels: Query<(), With<TerrainVoxel>>,
    cameras: Query<&GlobalTransform, With<Camera>>,
    _terrain_areas: Res<TerrainVisibleAreas>,
) {
    // if terrain_areas.is_changed() {
    //     for single_area in terrain_areas.get_all_current_visible_area() {
    //         info!(
    //             "current area: {:?} => {:?}",
    //             single_area.cached_min_voxle_graded_coord.chunk_coord,
    //             single_area.cached_max_voxle_graded_coord.chunk_coord
    //         )
    //     }
    //     for single_area in terrain_areas.get_all_last_visible_area() {
    //         info!(
    //             "last area: {:?} => {:?}",
    //             single_area.cached_min_voxle_graded_coord.chunk_coord,
    //             single_area.cached_max_voxle_graded_coord.chunk_coord
    //         )
    //     }
    // }

    info!(
        "cubes: {}, voxels: {}",
        cubes.iter().len(),
        voxels.iter().len()
    );

    for camera in cameras.iter() {
        info!("camera Location: {}", camera.translation());
    }
}
