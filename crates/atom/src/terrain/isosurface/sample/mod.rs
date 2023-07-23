use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::AsBindGroup,
};

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
        app.add_plugins(MaterialPlugin::<SamplerPointMaterial>::default())
            .add_systems(First, first_sample_surface)
            .add_systems(
                Update,
                init_surface_sampler.in_set(IsosurfaceExtractionSet::Sample),
            );
    }
}

fn first_sample_surface(
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

        info!(
            "world_offset: {}, voxel_size: {}, voxel_num: {}",
            world_offset, voxel_size, voxel_num
        );

        commands
            .entity(entity)
            .insert(SurfaceSampler::new(world_offset, voxel_size, voxel_num));
    }
}

fn init_surface_sampler(
    mut commands: Commands,
    mut surface_sampler_query: Query<(&mut SurfaceSampler, &mut IsosurfaceExtractionState)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<SamplerPointMaterial>>,
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

                // info!("value: {:?}", values);

                surface_sampler.set_sample_data(values);

                *state = IsosurfaceExtractionState::BuildOctree(BuildOctreeState::Build);
            }
        });

    for (surface_sampler, state) in surface_sampler_query.iter() {
        if *state == IsosurfaceExtractionState::BuildOctree(BuildOctreeState::Build) {
            commands.spawn(MaterialMeshBundle {
                mesh: meshes.add(Mesh::from(surface_sampler)),
                material: materials.add(SamplerPointMaterial::default()),
                ..Default::default()
            });
        }
    }
}

#[derive(Debug, Default, AsBindGroup, TypeUuid, TypePath, Clone)]
#[uuid = "00d75dd7-7431-465e-8072-b2162b82459f"]
struct SamplerPointMaterial {}

impl Material for SamplerPointMaterial {}
