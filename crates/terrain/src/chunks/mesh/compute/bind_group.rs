use bevy::{
    prelude::*,
    render::{
        render_resource::{BindGroup, BindGroupEntries},
        renderer::RenderDevice,
    },
};

use super::{buffer::TerrainChunkMeshBuffers, pipelines::TerrainChunkPipelines};

#[derive(Resource, Default)]
pub struct TerrainChunkBindGroups {
    pub main_mesh_bind_group: Option<BindGroup>,
}

pub struct TerrainChunkBindGroupsCreateContext<'a> {
    pub render_device: &'a RenderDevice,
    pub pipelines: &'a TerrainChunkPipelines,
    pub dynamic_buffers: &'a TerrainChunkMeshBuffers,
}

impl TerrainChunkBindGroups {
    pub fn create_bind_groups(&mut self, context: TerrainChunkBindGroupsCreateContext) {
        if context.dynamic_buffers.should_recreate {
            self.main_mesh_bind_group = Some(
                context.render_device.create_bind_group(
                    "terrain chunk main mesh bind group",
                    &context.pipelines.compute_bind_group_layout,
                    &BindGroupEntries::sequential((
                        context
                            .dynamic_buffers
                            .terrain_chunk_info_dynamic_buffer
                            .binding()
                            .expect("Terrain chunk info buffer binding should exist"),
                        context
                            .dynamic_buffers
                            .voxel_vertex_values_dynamic_buffer
                            .binding()
                            .expect("Voxel vertex values buffer binding should exist"),
                        context
                            .dynamic_buffers
                            .voxel_cross_points_dynamic_buffer
                            .binding()
                            .expect("Voxel cross points buffer binding should exist"),
                        context
                            .dynamic_buffers
                            .mesh_vertices_dynamic_buffer
                            .get_gpu_buffer()
                            .binding()
                            .expect("Mesh vertices GPU buffer binding should exist"),
                        context
                            .dynamic_buffers
                            .mesh_indices_dynamic_buffer
                            .get_gpu_buffer()
                            .binding()
                            .expect("Mesh indices GPU buffer binding should exist"),
                        context
                            .dynamic_buffers
                            .mesh_vertex_map_dynamic_buffer
                            .binding()
                            .expect("Mesh vertex map buffer binding should exist"),
                        context
                            .dynamic_buffers
                            .mesh_vertices_indices_count_dynamic_buffer
                            .get_gpu_buffer()
                            .binding()
                            .expect("Mesh vertices indices count GPU buffer binding should exist"),
                    )),
                ),
            );
        }
    }
}
