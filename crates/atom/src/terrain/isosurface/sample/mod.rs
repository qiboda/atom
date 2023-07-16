use bevy::prelude::*;

use crate::terrain::{
    chunk::{coords::TerrainChunkCoord, TerrainChunk},
    isosurface::BuildOctreeState,
    settings::TerrainSettings,
};

use self::surface_sampler::SurfaceSampler;

use super::{
    surface::shape_surface::ShapeSurface, IsosurfaceExtractionSet, IsosurfaceExtractionState,
};

pub mod sample_data;
pub mod surface_sampler;

pub struct SampleSurfacePlugin;

impl Plugin for SampleSurfacePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        info!("add SampleSurfacePlugin");
        app.add_systems(First, startup_sample_surface).add_systems(
            Update,
            init_surface_sampler.in_set(IsosurfaceExtractionSet::Sample),
        );
    }
}

fn startup_sample_surface(
    mut commands: Commands,
    terrain_settings: Res<TerrainSettings>,
    chunk_coord_query: Query<
        (Entity, &TerrainChunkCoord),
        (Without<SurfaceSampler>, With<TerrainChunk>),
    >,
) {
    // info!("startup_sample_surface: {:?}", chunk_coord_query);
    for (entity, chunk_coord) in chunk_coord_query.iter() {
        let voxel_num = UVec3::splat(terrain_settings.get_chunk_voxel_num());
        let voxel_size = Vec3::splat(terrain_settings.get_chunk_voxel_size());

        let world_offset = Vec3::new(
            chunk_coord.x as f32,
            chunk_coord.y as f32,
            chunk_coord.z as f32,
        ) * voxel_num.as_vec3()
            * voxel_size;

        commands
            .entity(entity)
            .insert(SurfaceSampler::new(world_offset, voxel_size, voxel_num));
    }
}

fn init_surface_sampler(
    mut surface_sampler_query: Query<(&mut SurfaceSampler, &mut IsosurfaceExtractionState)>,
    shape_surface: Res<ShapeSurface>,
) {
    surface_sampler_query
        .par_iter_mut()
        .for_each_mut(|(mut surface_sampler, mut state)| {
            if *state == IsosurfaceExtractionState::Sample {
                info!("init_surface_sampler");
                let offset = surface_sampler.world_offset;
                let size = surface_sampler.voxel_size * surface_sampler.get_sample_size().as_vec3();

                info!(
                    "range values: {:?}, {:?}, voxel_size: {:?}",
                    offset, size, surface_sampler.voxel_size
                );
                let values =
                    shape_surface.get_range_values(offset, size, surface_sampler.voxel_size);
                info!("sample value num: {}", values.len());

                let mut min_num = 0;
                let mut max_num = 0;
                let mut zero_num = 0;
                for i in values.iter() {
                    if i < &0.0 {
                        min_num += 1;
                    } else if i == &0.0 {
                        zero_num += 1;
                    } else {
                        max_num += 1;
                    }
                }

                info!(
                    "min num: {}, zero_num: {}, max_num: {}",
                    min_num, zero_num, max_num
                );

                info!("value: {:?}", values);

                surface_sampler.set_sample_data(values);
                info!(
                    "444 value: {}",
                    surface_sampler
                        .get_value_from_vertex_address(UVec3::new(4, 4, 4), &shape_surface)
                );

                info!(
                    "000 value: {}",
                    surface_sampler
                        .get_value_from_vertex_address(UVec3::new(0, 0, 0), &shape_surface)
                );

                info!(
                    "440 value: {}",
                    surface_sampler
                        .get_value_from_vertex_address(UVec3::new(4, 4, 0), &shape_surface)
                );

                info!(
                    "400 value: {}",
                    surface_sampler
                        .get_value_from_vertex_address(UVec3::new(4, 0, 0), &shape_surface)
                );

                *state = IsosurfaceExtractionState::BuildOctree(BuildOctreeState::Build);
            }
        });
}
