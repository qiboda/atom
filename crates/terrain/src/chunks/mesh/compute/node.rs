use bevy::{
    ecs::system::lifetimeless::Read,
    prelude::*,
    render::{
        render_graph::{self, RenderLabel},
        render_resource::{ComputePassDescriptor, PipelineCache},
    },
};

use crate::{
    chunks::{
        chunk::{TerrainChunkCoord, TerrainChunkLod},
        mesh::{TerrainChunkMeshingState, compute::buffer::TerrainChunkMeshBuffers},
    },
    terrain::setting::TerrainSetting,
};

use super::bind_group::TerrainChunkBindGroups;
use super::{buffer::DynamicBufferBindingInfo, pipelines::TerrainChunkPipelines};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct TerrainChunkMeshComputeLabel;

pub(crate) struct TerrainChunkMeshComputeNode {
    pub(crate) query: QueryState<(
        Entity,
        Read<TerrainChunkMeshingState>,
        Read<TerrainChunkCoord>,
        Read<TerrainChunkLod>,
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
        for (entity, state, _, _) in self.query.iter(world) {
            if *state == TerrainChunkMeshingState::Meshing
                || *state == TerrainChunkMeshingState::Seaming
            {
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

                if let Ok((entity, state, coord, _lod)) = self.query.get_manual(world, *entity)
                    && *state == TerrainChunkMeshingState::Meshing
                {
                    let _span = info_span!("TerrainChunkMeshComputeNode::run one main").entered();

                    debug!("main mesh node run: address: {:?}", coord);
                    let voxel_num = terrain_setting.get_voxel_count_in_chunk();
                    let workgroup_size = voxel_num / 4;

                    let dynamic_offset_mesh = mesh_buffers.get_buffers_dynamic_offset(entity);

                    {
                        let _span =
                            info_span!("TerrainChunkMeshComputeNode::run one main voxel vertex")
                                .entered();

                        let pipeline = pipeline_cache
                            .get_compute_pipeline(pipelines.compute_voxel_vertex_values_pipeline)
                            .expect("Failed to get compute voxel vertex values pipeline");
                        pass.set_bind_group(
                            0,
                            main_bind_groups
                                .main_mesh_bind_group
                                .as_ref()
                                .expect("Failed to get main mesh bind group"),
                            dynamic_offset_mesh.as_slice(),
                        );
                        pass.set_pipeline(pipeline);
                        pass.dispatch_workgroups(
                            workgroup_size + 1,
                            workgroup_size + 1,
                            workgroup_size + 1,
                        );
                    }

                    {
                        let _span = info_span!(
                            "TerrainChunkMeshComputeNode::run one main voxel cross points"
                        )
                        .entered();
                        let pipeline = pipeline_cache
                            .get_compute_pipeline(pipelines.compute_voxel_cross_points_pipeline)
                            .expect("Failed to get compute voxel cross points pipeline");
                        pass.set_bind_group(
                            0,
                            main_bind_groups
                                .main_mesh_bind_group
                                .as_ref()
                                .expect("Failed to get main mesh bind group"),
                            dynamic_offset_mesh.as_slice(),
                        );
                        pass.set_pipeline(pipeline);
                        pass.dispatch_workgroups(
                            workgroup_size + 1,
                            workgroup_size + 1,
                            workgroup_size + 1,
                        );
                    }

                    {
                        let _span =
                            info_span!("TerrainChunkMeshComputeNode::run one main mesh vertices")
                                .entered();

                        let pipeline = pipeline_cache
                            .get_compute_pipeline(pipelines.compute_vertices_pipeline)
                            .expect("Failed to get compute main vertices pipeline");
                        pass.set_bind_group(
                            0,
                            main_bind_groups
                                .main_mesh_bind_group
                                .as_ref()
                                .expect("Failed to get main mesh bind group"),
                            dynamic_offset_mesh.as_slice(),
                        );
                        pass.set_pipeline(pipeline);
                        pass.dispatch_workgroups(workgroup_size, workgroup_size, workgroup_size);
                    }

                    {
                        let _span =
                            info_span!("TerrainChunkMeshComputeNode::run one main mesh indices")
                                .entered();

                        let pipeline = pipeline_cache
                            .get_compute_pipeline(pipelines.compute_indices_pipeline)
                            .expect("Failed to get compute main indices pipeline");
                        pass.set_bind_group(
                            0,
                            main_bind_groups
                                .main_mesh_bind_group
                                .as_ref()
                                .expect("Failed to get main mesh bind group"),
                            dynamic_offset_mesh.as_slice(),
                        );
                        pass.set_pipeline(pipeline);
                        pass.dispatch_workgroups(workgroup_size, workgroup_size, workgroup_size);
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

            let mut max_vertices_indices_count_binding_info = DynamicBufferBindingInfo::default();
            let mut max_vertices_binding_info = DynamicBufferBindingInfo::default();
            let mut max_indices_binding_info = DynamicBufferBindingInfo::default();
            for (_, value) in mesh_buffers.terrain_chunk_buffer_bindings_map.iter() {
                if max_vertices_indices_count_binding_info.offset
                    < value.mesh_vertices_indices_count_buffer_binding.offset
                    || (max_vertices_indices_count_binding_info.offset
                        == value.mesh_vertices_indices_count_buffer_binding.offset
                        && max_vertices_indices_count_binding_info.size
                            < value.mesh_vertices_indices_count_buffer_binding.size)
                {
                    max_vertices_indices_count_binding_info =
                        value.mesh_vertices_indices_count_buffer_binding;
                }
                if max_vertices_binding_info.offset < value.mesh_vertices_buffer_binding.offset
                    || (max_vertices_binding_info.offset
                        == value.mesh_vertices_buffer_binding.offset
                        && max_vertices_binding_info.size < value.mesh_vertices_buffer_binding.size)
                {
                    max_vertices_binding_info = value.mesh_vertices_buffer_binding;
                }
                if max_indices_binding_info.offset < value.mesh_indices_buffer_binding.offset
                    || (max_indices_binding_info.offset == value.mesh_indices_buffer_binding.offset
                        && max_indices_binding_info.size < value.mesh_indices_buffer_binding.size)
                {
                    max_indices_binding_info = value.mesh_indices_buffer_binding;
                }
            }

            let command_encoder = render_context.command_encoder();

            mesh_buffers.mesh_vertices_dynamic_buffer.stage_buffer(
                command_encoder,
                max_vertices_binding_info.get_right_offset(),
            );
            mesh_buffers
                .mesh_indices_dynamic_buffer
                .stage_buffer(command_encoder, max_indices_binding_info.get_right_offset());
            mesh_buffers
                .mesh_vertices_indices_count_dynamic_buffer
                .stage_buffer(
                    command_encoder,
                    max_vertices_indices_count_binding_info.get_right_offset(),
                );
        }

        Ok(())
    }
}
