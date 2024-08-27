use std::ops::Not;

use bevy::{
    prelude::*,
    render::{
        render_resource::{BindGroup, BindGroupEntries, IntoBinding},
        renderer::RenderDevice,
        texture::GpuImage,
    },
};
use wgpu::{BindingResource, BufferBinding};

use super::{
    buffer_cache::{TerrainChunkMainDynamicBuffers, TerrainChunkMainRecreateBindGroup},
    pipelines::TerrainChunkPipelines,
};

use bevy::utils::HashMap;

#[derive(Resource, Default)]
pub struct TerrainChunkMainBindGroups {
    pub main_mesh_bind_group: Option<BindGroup>,
    /// key is csg operations number
    pub main_mesh_csg_bind_group: HashMap<u64, BindGroup>,

    pub main_mesh_map_bind_group: Option<BindGroup>,
}

pub struct TerrainChunkMainBindGroupsCreateContext<'a> {
    pub render_device: &'a RenderDevice,
    pub pipelines: &'a TerrainChunkPipelines,
    pub dynamic_buffers: &'a TerrainChunkMainDynamicBuffers,
    pub map_image: &'a GpuImage,
    pub map_biome_image: &'a GpuImage,
}

impl TerrainChunkMainBindGroups {
    pub fn create_bind_groups(&mut self, context: TerrainChunkMainBindGroupsCreateContext) {
        if context
            .dynamic_buffers
            .recreate_bind_group
            .contains(TerrainChunkMainRecreateBindGroup::MainMesh)
        {
            self.main_mesh_bind_group = Some(
                context.render_device.create_bind_group(
                    "terrain chunk main mesh bind group",
                    &context.pipelines.main_compute_bind_group_layout,
                    &BindGroupEntries::sequential((
                        context
                            .dynamic_buffers
                            .terrain_chunk_info_dynamic_buffer
                            .binding()
                            .unwrap(),
                        context
                            .dynamic_buffers
                            .voxel_vertex_values_dynamic_buffer
                            .binding()
                            .unwrap(),
                        context
                            .dynamic_buffers
                            .voxel_cross_points_dynamic_buffer
                            .binding()
                            .unwrap(),
                        context
                            .dynamic_buffers
                            .mesh_vertices_dynamic_buffer
                            .get_gpu_buffer()
                            .binding()
                            .unwrap(),
                        context
                            .dynamic_buffers
                            .mesh_indices_dynamic_buffer
                            .get_gpu_buffer()
                            .binding()
                            .unwrap(),
                        context
                            .dynamic_buffers
                            .mesh_vertex_map_dynamic_buffer
                            .binding()
                            .unwrap(),
                        context
                            .dynamic_buffers
                            .mesh_vertices_indices_count_dynamic_buffer
                            .get_gpu_buffer()
                            .binding()
                            .unwrap(),
                    )),
                ),
            );
        }

        if context
            .dynamic_buffers
            .recreate_bind_group
            .contains(TerrainChunkMainRecreateBindGroup::CSG)
        {
            self.main_mesh_csg_bind_group.clear();
        }

        for (_key, value) in context
            .dynamic_buffers
            .terrain_chunk_buffer_bindings_map
            .iter()
        {
            let csg_operations_binding = &value.csg_operations_buffer_binding;
            let size = csg_operations_binding.size.unwrap().get();
            if self.main_mesh_csg_bind_group.contains_key(&size).not() {
                let bind_group = context.render_device.create_bind_group(
                    "terrain chunk main mesh bind group",
                    &context.pipelines.main_compute_csg_bind_group_layout,
                    &BindGroupEntries::sequential((
                        context
                            .dynamic_buffers
                            .csg_info_dynamic_buffer
                            .binding()
                            .unwrap(),
                        BindingResource::Buffer(BufferBinding {
                            buffer: context
                                .dynamic_buffers
                                .csg_operations_dynamic_buffer
                                .buffer()
                                .unwrap(),
                            offset: 0,
                            size: csg_operations_binding.size,
                        }),
                    )),
                );
                self.main_mesh_csg_bind_group.insert(size, bind_group);
            }
        }

        if self.main_mesh_map_bind_group.is_none() {
            info!("create main_mesh_map_bind_group ok");
            self.main_mesh_map_bind_group = Some(
                context.render_device.create_bind_group(
                    "terrain chunk main mesh map bind group",
                    &context.pipelines.main_compute_map_bind_group_layout,
                    &BindGroupEntries::sequential((
                        context
                            .dynamic_buffers
                            .terrain_map_config_buffer
                            .as_ref()
                            .unwrap()
                            .into_binding(),
                        context.map_image.texture_view.into_binding(),
                        context.map_image.sampler.into_binding(),
                        context.map_biome_image.texture_view.into_binding(),
                        context.map_biome_image.sampler.into_binding(),
                    )),
                ),
            );
        }
    }

    pub fn get_csg_binding_group(&self, buff_size: u64) -> &BindGroup {
        self.main_mesh_csg_bind_group.get(&buff_size).unwrap()
    }
}
