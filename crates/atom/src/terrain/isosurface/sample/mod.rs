use bevy::prelude::*;

use crate::terrain::{
    chunk::{coords::TerrainChunkCoord, TerrainChunk},
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

                let values =
                    shape_surface.get_range_values(offset, size, surface_sampler.voxel_size);
                surface_sampler.set_sample_data(values);

                *state = IsosurfaceExtractionState::Extract;
            }
        });
}
