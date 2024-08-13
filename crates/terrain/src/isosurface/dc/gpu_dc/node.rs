use bevy::{
    ecs::system::lifetimeless::Read,
    prelude::*,
    render::{
        render_graph::{self, RenderLabel},
        render_resource::{ComputePassDescriptor, PipelineCache},
    },
};

use crate::{
    chunk_mgr::chunk::state::{TerrainChunkAddress, TerrainChunkSeamLod},
    isosurface::dc::gpu_dc::buffer_cache::TerrainChunkSeamKey,
    setting::TerrainSetting,
    tables::SubNodeIndex,
};

use super::bind_group_cache::TerrainChunkMainBindGroupCachedId;
use super::buffer_cache::TerrainChunkMainBuffersCache;
use super::pipelines::TerrainChunkPipelines;
use super::{
    bind_group_cache::TerrainChunkSeamBindGroupCachedId,
    buffer_cache::TerrainChunkMainBufferCachedId,
};
use super::{
    bind_group_cache::{TerrainChunkMainBindGroupsCache, TerrainChunkSeamBindGroupsCache},
    buffer_cache::{TerrainChunkSeamBufferCachedId, TerrainChunkSeamBuffersCache},
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct TerrainChunkMeshComputeLabel;

pub(crate) struct TerrainChunkMeshComputeNode {
    #[allow(clippy::type_complexity)]
    pub(crate) query: QueryState<(
        Entity,
        Read<TerrainChunkAddress>,
        Option<Read<TerrainChunkMainBufferCachedId>>,
        Option<Read<TerrainChunkMainBindGroupCachedId>>,
        Read<TerrainChunkSeamLod>,
        Option<Read<TerrainChunkSeamBufferCachedId>>,
        Option<Read<TerrainChunkSeamBindGroupCachedId>>,
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
        for (entity, _, _, _, _, _, _) in self.query.iter(world) {
            self.entities.push(entity);
        }
    }

    fn run<'w>(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        let _span = info_span!(
            "TerrainChunkMeshComputeNode::run",
            count = self.entities.len()
        )
        .entered();

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipelines = world.resource::<TerrainChunkPipelines>();
        let main_buffers_cache = world.resource::<TerrainChunkMainBuffersCache>();
        let main_bind_groups_cache = world.resource::<TerrainChunkMainBindGroupsCache>();
        let seam_buffers_cache = world.resource::<TerrainChunkSeamBuffersCache>();
        let seam_bind_groups_cache = world.resource::<TerrainChunkSeamBindGroupsCache>();
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

                if let Ok((_, address, _, main_bind_groups_id, seam_lod, _, seam_bind_group_id)) =
                    self.query.get_manual(world, *entity)
                {
                    if let Some(main_bind_groups_id) = main_bind_groups_id {
                        let _span =
                            info_span!("TerrainChunkMeshComputeNode::run one main").entered();

                        debug!("main mesh node run: address: {:?}", address);
                        let voxel_num = terrain_setting.get_voxel_num_in_chunk();
                        let workgroup_size = (voxel_num / 4) as u32;

                        let main_bind_groups = main_bind_groups_cache
                            .get_bind_groups(*main_bind_groups_id)
                            .unwrap();

                        {
                            let _span = info_span!(
                                "TerrainChunkMeshComputeNode::run one main voxel vertex"
                            )
                            .entered();

                            let pipeline = pipeline_cache
                                .get_compute_pipeline(
                                    pipelines.compute_voxel_vertex_values_pipeline,
                                )
                                .unwrap();
                            pass.set_bind_group(0, &main_bind_groups.main_mesh_bind_group, &[]);
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
                                .unwrap();
                            pass.set_bind_group(0, &main_bind_groups.main_mesh_bind_group, &[]);
                            pass.set_pipeline(pipeline);
                            pass.dispatch_workgroups(
                                workgroup_size + 1,
                                workgroup_size + 1,
                                workgroup_size + 1,
                            );
                        }

                        {
                            let _span = info_span!(
                                "TerrainChunkMeshComputeNode::run one main mesh vertices"
                            )
                            .entered();

                            let pipeline = pipeline_cache
                                .get_compute_pipeline(pipelines.main_compute_vertices_pipeline)
                                .unwrap();
                            pass.set_bind_group(0, &main_bind_groups.main_mesh_bind_group, &[]);
                            pass.set_pipeline(pipeline);
                            pass.dispatch_workgroups(
                                workgroup_size,
                                workgroup_size,
                                workgroup_size,
                            );
                        }

                        {
                            let _span = info_span!(
                                "TerrainChunkMeshComputeNode::run one main mesh indices"
                            )
                            .entered();

                            let pipeline = pipeline_cache
                                .get_compute_pipeline(pipelines.main_compute_indices_pipeline)
                                .unwrap();
                            pass.set_bind_group(0, &main_bind_groups.main_mesh_bind_group, &[]);
                            pass.set_pipeline(pipeline);
                            pass.dispatch_workgroups(
                                workgroup_size,
                                workgroup_size,
                                workgroup_size,
                            );
                        }
                    }

                    if let Some(seam_bind_groups_id) = seam_bind_group_id {
                        let _span =
                            info_span!("TerrainChunkMeshComputeNode::run one seam").entered();
                        debug!("seam mesh node run: address: {:?}", address);

                        let max_lod = seam_lod.get_max_lod();
                        // let voxel_num =
                        //     terrain_setting.get_voxel_num_in_chunk() * 2usize.pow(max_lod as u32);
                        let add_lod = seam_lod.get_lod(SubNodeIndex::X0Y0Z0);
                        let level = address.0.level();
                        let voxel_size = terrain_setting.get_voxel_size(level + add_lod[0]);
                        let chunk_size = terrain_setting.get_chunk_size(level);

                        let voxel_num = (chunk_size / voxel_size).round();
                        let workgroup_size = voxel_num as u32;

                        let terrain_chunk_seam_key = TerrainChunkSeamKey { lod: *seam_lod };

                        debug!( "Terrain Seam info: address: {:?}, voxel_num: {}, max lod: {}, workgroup size:{}", address, voxel_num, max_lod, workgroup_size);

                        let x_axis_bind_groups = seam_bind_groups_cache
                            .get_bind_groups(terrain_chunk_seam_key, seam_bind_groups_id[0])
                            .unwrap();

                        // x axis
                        {
                            {
                                let _span = info_span!(
                                    "TerrainChunkMeshComputeNode::run one seam x axis vertices"
                                )
                                .entered();
                                let pipeline = pipeline_cache
                                    .get_compute_pipeline(
                                        pipelines.seam_compute_vertices_x_axis_pipeline,
                                    )
                                    .unwrap();
                                pass.set_bind_group(
                                    0,
                                    &x_axis_bind_groups.seam_mesh_bind_group,
                                    &[],
                                );
                                pass.set_pipeline(pipeline);
                                pass.dispatch_workgroups(1, workgroup_size + 1, workgroup_size + 1);
                            }

                            {
                                let _span = info_span!(
                                    "TerrainChunkMeshComputeNode::run one seam x axis indices"
                                )
                                .entered();
                                let pipeline = pipeline_cache
                                    .get_compute_pipeline(
                                        pipelines.seam_compute_indices_x_axis_pipeline,
                                    )
                                    .unwrap();
                                pass.set_bind_group(
                                    0,
                                    &x_axis_bind_groups.seam_mesh_bind_group,
                                    &[],
                                );
                                pass.set_pipeline(pipeline);
                                pass.dispatch_workgroups(1, workgroup_size + 1, workgroup_size + 1);
                            }
                        }

                        let y_axis_bind_groups = seam_bind_groups_cache
                            .get_bind_groups(terrain_chunk_seam_key, seam_bind_groups_id[1])
                            .unwrap();

                        // y axis
                        {
                            {
                                let _span = info_span!(
                                    "TerrainChunkMeshComputeNode::run one seam y axis vertices"
                                )
                                .entered();
                                let pipeline = pipeline_cache
                                    .get_compute_pipeline(
                                        pipelines.seam_compute_vertices_y_axis_pipeline,
                                    )
                                    .unwrap();
                                pass.set_bind_group(
                                    0,
                                    &y_axis_bind_groups.seam_mesh_bind_group,
                                    &[],
                                );
                                pass.set_pipeline(pipeline);
                                pass.dispatch_workgroups(workgroup_size + 1, 1, workgroup_size + 1);
                            }
                            {
                                let _span = info_span!(
                                    "TerrainChunkMeshComputeNode::run one seam y axis indices"
                                )
                                .entered();
                                let pipeline = pipeline_cache
                                    .get_compute_pipeline(
                                        pipelines.seam_compute_indices_y_axis_pipeline,
                                    )
                                    .unwrap();
                                pass.set_bind_group(
                                    0,
                                    &y_axis_bind_groups.seam_mesh_bind_group,
                                    &[],
                                );
                                pass.set_pipeline(pipeline);
                                pass.dispatch_workgroups(workgroup_size + 1, 1, workgroup_size + 1);
                            }
                        }

                        let z_axis_bind_groups = seam_bind_groups_cache
                            .get_bind_groups(terrain_chunk_seam_key, seam_bind_groups_id[2])
                            .unwrap();

                        // z axis
                        {
                            {
                                let _span = info_span!(
                                    "TerrainChunkMeshComputeNode::run one seam z axis vertices"
                                )
                                .entered();
                                let pipeline = pipeline_cache
                                    .get_compute_pipeline(
                                        pipelines.seam_compute_vertices_z_axis_pipeline,
                                    )
                                    .unwrap();
                                pass.set_bind_group(
                                    0,
                                    &z_axis_bind_groups.seam_mesh_bind_group,
                                    &[],
                                );
                                pass.set_pipeline(pipeline);
                                pass.dispatch_workgroups(workgroup_size + 1, workgroup_size + 1, 1);
                            }

                            {
                                let _span = info_span!(
                                    "TerrainChunkMeshComputeNode::run one seam z axis indices"
                                )
                                .entered();
                                let pipeline = pipeline_cache
                                    .get_compute_pipeline(
                                        pipelines.seam_compute_indices_z_axis_pipeline,
                                    )
                                    .unwrap();
                                pass.set_bind_group(
                                    0,
                                    &z_axis_bind_groups.seam_mesh_bind_group,
                                    &[],
                                );
                                pass.set_pipeline(pipeline);
                                pass.dispatch_workgroups(workgroup_size + 1, workgroup_size + 1, 1);
                            }
                        }
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

            let command_encoder = render_context.command_encoder();
            for entity in self.entities.iter() {
                let _span =
                    info_span!("TerrainChunkMeshComputeNode::run one stage buffers").entered();

                if let Ok((_, _, main_buffers_id, _, seam_lod, seam_buffers_id, _)) =
                    self.query.get_manual(world, *entity)
                {
                    if let Some(main_buffers_id) = main_buffers_id {
                        let _span =
                            info_span!("TerrainChunkMeshComputeNode::run one main stage buffers")
                                .entered();
                        let buffers = main_buffers_cache.get_buffers(*main_buffers_id).unwrap();
                        buffers.stage_buffers(command_encoder);
                    }

                    if let Some(seam_buffers_id) = seam_buffers_id {
                        let _span =
                            info_span!("TerrainChunkMeshComputeNode::run one seam stage buffers")
                                .entered();
                        let terrain_chunk_seam_key = TerrainChunkSeamKey { lod: *seam_lod };
                        let x_axis_buffers = seam_buffers_cache
                            .get_buffers(terrain_chunk_seam_key, seam_buffers_id[0])
                            .unwrap();
                        let y_axis_buffers = seam_buffers_cache
                            .get_buffers(terrain_chunk_seam_key, seam_buffers_id[1])
                            .unwrap();
                        let z_axis_buffers = seam_buffers_cache
                            .get_buffers(terrain_chunk_seam_key, seam_buffers_id[2])
                            .unwrap();
                        x_axis_buffers.stage_buffers(command_encoder);
                        y_axis_buffers.stage_buffers(command_encoder);
                        z_axis_buffers.stage_buffers(command_encoder);
                    }
                }
            }
        }

        Ok(())
    }
}
