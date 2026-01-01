use bevy::{
    ecs::system::lifetimeless::Read,
    prelude::*,
    render::{
        render_graph::{self, RenderLabel},
        render_resource::{CachedPipelineState, ComputePassDescriptor, PipelineCache},
    },
};

use crate::{
    chunks::{
        chunk::TerrainChunkCoord,
        mesh::{components::TerrainChunkMeshingState, compute::buffer::TerrainChunkMeshBuffers},
    },
    terrain::setting::TerrainSetting,
};

use super::bind_group::TerrainChunkBindGroups;
use super::pipelines::TerrainChunkPipelines;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct TerrainChunkMeshComputeLabel;

pub(crate) struct TerrainChunkMeshComputeNode {
    pub(crate) query: QueryState<(
        Entity,
        Read<TerrainChunkMeshingState>,
        Read<TerrainChunkCoord>,
    )>,
    pub(crate) entities: Vec<Entity>,
}

impl FromWorld for TerrainChunkMeshComputeNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
            entities: Vec::new(),
        }
    }
}

impl render_graph::Node for TerrainChunkMeshComputeNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);

        self.entities.clear();
        for (entity, state, _) in self.query.iter(world) {
            if *state == TerrainChunkMeshingState::Meshing {
                self.entities.push(entity);
            }
        }
    }

    fn run<'w>(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        if self.entities.is_empty() {
            return Ok(());
        }

        let _span = info_span!(
            "TerrainChunkMeshComputeNode::run",
            count = self.entities.len()
        )
        .entered();

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipelines = world.resource::<TerrainChunkPipelines>();
        let mesh_buffers = world.resource::<TerrainChunkMeshBuffers>();
        let main_bind_groups = world.resource::<TerrainChunkBindGroups>();
        let terrain_setting = world.resource::<TerrainSetting>();

        // 提前检查所有需要的管道是否准备好
        let Some(voxel_vertex_pipeline) =
            pipeline_cache.get_compute_pipeline(pipelines.compute_voxel_vertex_values_pipeline)
        else {
            if let CachedPipelineState::Err(pipeline_cache_error) = pipeline_cache
                .get_compute_pipeline_state(pipelines.compute_voxel_vertex_values_pipeline)
            {
                error!(
                    "Failed to get voxel vertex compute pipeline: {:?}",
                    pipeline_cache_error
                );
            }
            return Ok(()); // 管道未准备好，跳过整个节点
        };
        let Some(cross_points_pipeline) =
            pipeline_cache.get_compute_pipeline(pipelines.compute_voxel_cross_points_pipeline)
        else {
            if let CachedPipelineState::Err(pipeline_cache_error) = pipeline_cache
                .get_compute_pipeline_state(pipelines.compute_voxel_cross_points_pipeline)
            {
                error!(
                    "Failed to get voxel cross points compute pipeline: {:?}",
                    pipeline_cache_error
                );
            }
            return Ok(());
        };

        let Some(vertices_pipeline) =
            pipeline_cache.get_compute_pipeline(pipelines.compute_vertices_pipeline)
        else {
            if let CachedPipelineState::Err(pipeline_cache_error) =
                pipeline_cache.get_compute_pipeline_state(pipelines.compute_vertices_pipeline)
            {
                error!(
                    "Failed to get mesh vertices compute pipeline: {:?}",
                    pipeline_cache_error
                );
            }
            return Ok(());
        };

        let Some(indices_pipeline) =
            pipeline_cache.get_compute_pipeline(pipelines.compute_indices_pipeline)
        else {
            if let CachedPipelineState::Err(pipeline_cache_error) =
                pipeline_cache.get_compute_pipeline_state(pipelines.compute_indices_pipeline)
            {
                error!(
                    "Failed to get mesh indices compute pipeline: {:?}",
                    pipeline_cache_error
                );
            }
            return Ok(());
        };

        {
            let _span = info_span!(
                "TerrainChunkMeshComputeNode::run compute",
                count = self.entities.len()
            )
            .entered();

            let command_encoder = render_context.command_encoder();
            let mut pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
                label: "terrain chunk mesh compute pass".into(),
                timestamp_writes: None,
            });

            for entity in self.entities.iter() {
                let _span = info_span!("TerrainChunkMeshComputeNode::run one").entered();

                if let Ok((entity, state, coord)) = self.query.get_manual(world, *entity) {
                    assert!(*state == TerrainChunkMeshingState::Meshing);

                    let _span = info_span!("TerrainChunkMeshComputeNode::run one main").entered();

                    debug!("main mesh node run: coord: {:?}", coord);
                    let voxel_num = terrain_setting.get_voxel_count_in_compute();
                    // 为什么除以4, 因为shader中是以4为一个workgroup来处理的。
                    // TODO: 先用4看看效果，之后可以测试看看2怎么样。
                    let voxel_vertex_or_edge_workgroup_size = (voxel_num + 1) / 4;
                    let voxel_workgroup_size = voxel_num / 4;

                    let dynamic_offset_mesh = mesh_buffers.get_buffers_dynamic_offset(entity);

                    {
                        let _span =
                            info_span!("TerrainChunkMeshComputeNode::run one main voxel vertex")
                                .entered();

                        pass.set_bind_group(
                            0,
                            main_bind_groups
                                .main_mesh_bind_group
                                .as_ref()
                                .expect("Failed to get main mesh bind group"),
                            dynamic_offset_mesh.as_slice(),
                        );
                        pass.set_pipeline(voxel_vertex_pipeline);
                        pass.dispatch_workgroups(
                            voxel_vertex_or_edge_workgroup_size,
                            voxel_vertex_or_edge_workgroup_size,
                            voxel_vertex_or_edge_workgroup_size,
                        );
                    }

                    {
                        let _span = info_span!(
                            "TerrainChunkMeshComputeNode::run one main voxel cross points"
                        )
                        .entered();
                        pass.set_bind_group(
                            0,
                            main_bind_groups
                                .main_mesh_bind_group
                                .as_ref()
                                .expect("Failed to get main mesh bind group"),
                            dynamic_offset_mesh.as_slice(),
                        );
                        pass.set_pipeline(cross_points_pipeline);
                        pass.dispatch_workgroups(
                            voxel_vertex_or_edge_workgroup_size,
                            voxel_vertex_or_edge_workgroup_size,
                            voxel_vertex_or_edge_workgroup_size,
                        );
                    }

                    {
                        let _span =
                            info_span!("TerrainChunkMeshComputeNode::run one main mesh vertices")
                                .entered();

                        pass.set_bind_group(
                            0,
                            main_bind_groups
                                .main_mesh_bind_group
                                .as_ref()
                                .expect("Failed to get main mesh bind group"),
                            dynamic_offset_mesh.as_slice(),
                        );
                        pass.set_pipeline(vertices_pipeline);
                        pass.dispatch_workgroups(
                            voxel_workgroup_size,
                            voxel_workgroup_size,
                            voxel_workgroup_size,
                        );
                    }

                    {
                        let _span =
                            info_span!("TerrainChunkMeshComputeNode::run one main mesh indices")
                                .entered();

                        pass.set_bind_group(
                            0,
                            main_bind_groups
                                .main_mesh_bind_group
                                .as_ref()
                                .expect("Failed to get main mesh bind group"),
                            dynamic_offset_mesh.as_slice(),
                        );
                        pass.set_pipeline(indices_pipeline);
                        pass.dispatch_workgroups(
                            voxel_workgroup_size,
                            voxel_workgroup_size,
                            voxel_workgroup_size,
                        );
                    }
                }
            }
        }

        {
            let _span = info_span!(
                "TerrainChunkMeshComputeNode::run stage all",
                count = self.entities.len()
            )
            .entered();

            // 获取需要stage的右边界，并 stage buffer
            let mut max_vertices_indices_count_right_offset = 0;
            let mut max_vertices_right_offset = 0;
            let mut max_indices_right_offset = 0;
            for (_, value) in mesh_buffers.terrain_chunk_buffer_bindings_map.iter() {
                if max_vertices_indices_count_right_offset
                    < value
                        .mesh_vertices_indices_count_buffer_binding
                        .get_right_offset()
                {
                    max_vertices_indices_count_right_offset = value
                        .mesh_vertices_indices_count_buffer_binding
                        .get_right_offset();
                }
                if max_vertices_right_offset < value.mesh_vertices_buffer_binding.get_right_offset()
                {
                    max_vertices_right_offset =
                        value.mesh_vertices_buffer_binding.get_right_offset();
                }
                if max_indices_right_offset < value.mesh_indices_buffer_binding.get_right_offset() {
                    max_indices_right_offset = value.mesh_indices_buffer_binding.get_right_offset();
                }
            }

            let command_encoder = render_context.command_encoder();

            mesh_buffers
                .mesh_vertices_buffer
                .stage_buffer(command_encoder, max_vertices_right_offset);
            mesh_buffers
                .mesh_indices_buffer
                .stage_buffer(command_encoder, max_indices_right_offset);
            mesh_buffers
                .mesh_vertices_indices_count_buffer
                .stage_buffer(command_encoder, max_vertices_indices_count_right_offset);
        }

        Ok(())
    }
}
