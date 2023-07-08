use bevy::prelude::*;

use crate::terrain::data::{coords::TerrainChunkCoord, settings::TerrainSettings, TerrainChunk};

use self::surface_sampler::SurfaceSampler;

use super::{IsosurfaceExtract, IsosurfaceExtractionSet};

pub mod sample_range_3d;
pub mod surface_sampler;

pub struct SampleSurfacePlugin;

impl Plugin for SampleSurfacePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(
            Startup,
            (startup_sample_surface).in_set(IsosurfaceExtractionSet::Sample),
        );
    }
}

fn startup_sample_surface(
    mut commands: Commands,
    terrain_settings: Res<TerrainSettings>,
    chunk_coord_query: Query<(&Children, &TerrainChunkCoord), With<TerrainChunk>>,
    _children: Query<Entity, With<IsosurfaceExtract>>,
) {
    for (children, chunk_coord) in chunk_coord_query.iter() {
        for child in children.iter() {
            let sample_size = UVec3::splat(terrain_settings.get_chunk_voxel_num());
            let voxel_size = Vec3::splat(terrain_settings.get_chunk_voxel_size());

            let _world_offset = Vec3::new(
                chunk_coord.x as f32,
                chunk_coord.y as f32,
                chunk_coord.z as f32,
            ) * sample_size.as_vec3()
                * voxel_size;
            commands
                .spawn(SurfaceSampler::new(Vec3::ZERO, voxel_size))
                .set_parent(*child);
        }
    }
}
