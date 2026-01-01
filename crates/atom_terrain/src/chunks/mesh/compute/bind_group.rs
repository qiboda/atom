use bevy::{
    prelude::*,
    render::{
        render_resource::{BindGroup, BindGroupEntries},
        renderer::RenderDevice,
    },
};

use crate::chunks::{chunk::TerrainChunkCoord, mesh::components::TerrainChunkMeshingState};

use super::{buffer::TerrainChunkMeshBuffers, pipelines::TerrainChunkPipelines};

#[derive(Resource, Default)]
pub struct TerrainChunkBindGroups {
    pub main_mesh_bind_group: Option<BindGroup>,
}

pub struct TerrainChunkBindGroupsCreateContext<'a> {
    pub render_device: &'a RenderDevice,
    pub pipelines: &'a TerrainChunkPipelines,
    pub mesh_buffers: &'a mut TerrainChunkMeshBuffers,
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
    }
}

pub(crate) fn prepare_mesh_bind_group(
    pipelines: Res<TerrainChunkPipelines>,
    render_device: Res<RenderDevice>,
    query: Query<(Entity, &TerrainChunkMeshingState, &TerrainChunkCoord)>,
    mut bind_groups: ResMut<TerrainChunkBindGroups>,
    mut mesh_buffers: ResMut<TerrainChunkMeshBuffers>,
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

    let context = TerrainChunkBindGroupsCreateContext {
        render_device: &render_device,
        pipelines: &pipelines,
        mesh_buffers: &mut mesh_buffers,
    };
    bind_groups.create_bind_groups(context);
}
