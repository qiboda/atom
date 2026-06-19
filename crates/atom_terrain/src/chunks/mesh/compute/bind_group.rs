use bevy::{
    prelude::*,
    render::{
        render_resource::{
            BindGroup, BindGroupEntries, BufferInitDescriptor, BufferUsages, SamplerDescriptor,
        },
        render_asset::RenderAssets,
        renderer::RenderDevice,
        texture::GpuImage,
    },
};

use crate::chunks::{
    chunk::TerrainChunkCoord,
    mesh::components::TerrainChunkMeshingState,
};

use super::{buffer::TerrainChunkMeshBuffers, pipelines::TerrainChunkPipelines, types::TerrainMapConfig};

#[derive(Resource, Default)]
pub struct TerrainChunkBindGroups {
    pub main_mesh_bind_group: Option<BindGroup>,
    pub map_bind_group: Option<BindGroup>,
}

pub struct TerrainChunkBindGroupsCreateContext<'a> {
    pub render_device: &'a RenderDevice,
    pub pipelines: &'a TerrainChunkPipelines,
    pub mesh_buffers: &'a mut TerrainChunkMeshBuffers,
    pub biome_image: Option<&'a GpuImage>,
    pub terrain_map_config: &'a TerrainMapConfig,
}

impl TerrainChunkBindGroups {
    pub fn create_bind_groups(&mut self, context: TerrainChunkBindGroupsCreateContext) {
        if context.mesh_buffers.should_create {
            info!("Creating terrain chunk main mesh bind group");
            self.main_mesh_bind_group = Some(
                context.render_device.create_bind_group(
                    "terrain chunk main mesh bind group",
                    &context.pipelines.compute_bind_group_layout,
                    &BindGroupEntries::sequential((
                        context
                            .mesh_buffers
                            .terrain_chunk_info_buffer
                            .binding()
                            .expect("Terrain chunk info buffer binding should exist"),
                        context
                            .mesh_buffers
                            .voxel_vertex_values_buffer
                            .binding()
                            .expect("Voxel vertex values buffer binding should exist"),
                        context
                            .mesh_buffers
                            .voxel_cross_points_buffer
                            .binding()
                            .expect("Voxel cross points buffer binding should exist"),
                        context
                            .mesh_buffers
                            .mesh_vertices_buffer
                            .get_gpu_buffer()
                            .binding()
                            .expect("Mesh vertices GPU buffer binding should exist"),
                        context
                            .mesh_buffers
                            .mesh_indices_buffer
                            .get_gpu_buffer()
                            .binding()
                            .expect("Mesh indices GPU buffer binding should exist"),
                        context
                            .mesh_buffers
                            .mesh_vertex_map_buffer
                            .binding()
                            .expect("Mesh vertex map buffer binding should exist"),
                        context
                            .mesh_buffers
                            .mesh_vertices_indices_count_buffer
                            .get_gpu_buffer()
                            .binding()
                            .expect("Mesh vertices indices count GPU buffer binding should exist"),
                    )),
                ),
            );
            context.mesh_buffers.should_create = false;
        }

        // 创建地图 bind group (group 1): TerrainMapConfig + biome_texture + sampler
        if let Some(biome_image) = context.biome_image {
            if self.map_bind_group.is_none() {
                info!("Creating terrain chunk map bind group");
                let biome_sampler = context
                    .render_device
                    .create_sampler(&SamplerDescriptor {
                        label: Some("biome map sampler"),
                        ..default()
                    });
                let biome_view = biome_image.texture.create_view(&Default::default());

                let config_buffer = context.render_device.create_buffer_with_data(
                    &BufferInitDescriptor {
                        label: Some("terrain map config buffer"),
                        contents: bytemuck::bytes_of(context.terrain_map_config),
                        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                    },
                );

                self.map_bind_group = Some(context.render_device.create_bind_group(
                    "terrain chunk map bind group",
                    &context.pipelines.compute_map_bind_group_layout,
                    &BindGroupEntries::sequential((
                        config_buffer.as_entire_binding(),
                        bevy::render::render_resource::BindingResource::TextureView(&biome_view),
                        bevy::render::render_resource::BindingResource::Sampler(&biome_sampler),
                    )),
                ));
            }
        }
    }
}

pub(crate) fn prepare_mesh_bind_group(
    pipelines: Res<TerrainChunkPipelines>,
    render_device: Res<RenderDevice>,
    query: Query<(Entity, &TerrainChunkMeshingState, &TerrainChunkCoord)>,
    mut bind_groups: ResMut<TerrainChunkBindGroups>,
    mut mesh_buffers: ResMut<TerrainChunkMeshBuffers>,
    terrain_region_info: Option<Res<crate::biomes::generator::TerrainRegionInfo>>,
    gpu_images: Res<RenderAssets<GpuImage>>,
) {
    let mut chunk_meshing_count = 0;
    for (_entity, state, _coord) in query.iter() {
        if *state != TerrainChunkMeshingState::Meshing {
            continue;
        }
        chunk_meshing_count += 1;
    }

    if chunk_meshing_count == 0 {
        return;
    }

    let biome_image = terrain_region_info
        .as_ref()
        .and_then(|info| gpu_images.get(&info.biome_image));

    let terrain_map_config = TerrainMapConfig {
        terrain_height: 4096.0,
        pixel_size: 2.0,
        temperature_min: -10.0,
        temperature_max: 40.0,
    };

    let context = TerrainChunkBindGroupsCreateContext {
        render_device: &render_device,
        pipelines: &pipelines,
        mesh_buffers: &mut mesh_buffers,
        biome_image,
        terrain_map_config: &terrain_map_config,
    };
    bind_groups.create_bind_groups(context);
}
