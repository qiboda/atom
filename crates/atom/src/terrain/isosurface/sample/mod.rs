use bevy::prelude::*;

use crate::terrain::chunk::{coords::TerrainChunkCoord, settings::TerrainSettings, TerrainChunk};

use self::surface_sampler::SurfaceSampler;

use super::{surface::shape_surface::ShapeSurface, IsosurfaceExtractionSet};

pub mod sample_data;
pub mod surface_sampler;

pub struct SampleSurfacePlugin;

impl Plugin for SampleSurfacePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Startup,
            (startup_sample_surface, init_surface_sampler)
                .chain()
                .in_set(IsosurfaceExtractionSet::Initialize),
        );
    }
}

fn startup_sample_surface(
    mut commands: Commands,
    terrain_settings: Res<TerrainSettings>,
    chunk_coord_query: Query<(&Children, &TerrainChunkCoord), With<TerrainChunk>>,
) {
    for (children, chunk_coord) in chunk_coord_query.iter() {
        for child in children.iter() {
            let voxel_num = UVec3::splat(terrain_settings.get_chunk_voxel_num());
            let voxel_size = Vec3::splat(terrain_settings.get_chunk_voxel_size());

            let world_offset = Vec3::new(
                chunk_coord.x as f32,
                chunk_coord.y as f32,
                chunk_coord.z as f32,
            ) * voxel_num.as_vec3()
                * voxel_size;

            commands
                .spawn(SurfaceSampler::new(world_offset, voxel_size, voxel_num))
                .set_parent(*child);
        }
    }
}

fn init_surface_sampler(
    mut surface_sampler_query: Query<&mut SurfaceSampler>,
    shape_surface: Res<ShapeSurface>,
) {
    for mut surface_sampler in surface_sampler_query.iter_mut() {
        let offset = surface_sampler.world_offset;
        let size = surface_sampler.voxel_size * surface_sampler.get_sample_size().as_vec3();

        let values = shape_surface.get_range_values(offset, size, surface_sampler.voxel_size);
        surface_sampler.set_sample_data(values);
    }
}
