use bevy::{
    diagnostic::FrameCount,
    ecs::system::lifetimeless::{Read, Write},
    prelude::*,
    render::{
        render_graph::{self, RenderLabel},
        render_resource::{CachedPipelineState, ComputePassDescriptor, PipelineCache},
    },
};

use crate::{
    chunks::{
        chunk::TerrainChunkCoord,
        mesh::{
            components::TerrainChunkMeshingState,
            compute::{buffer::TerrainChunkMeshBuffers, mesh_compute::TerrainChunkComputeState},
        },
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
        Write<TerrainChunkComputeState>,
    )>,
    pub(crate) to_compute_entities: Vec<Entity>,
}

impl FromWorld for TerrainChunkMeshComputeNode {
    fn from_world(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
            to_compute_entities: Vec::new(),
        }
    }
}

impl render_graph::Node for TerrainChunkMeshComputeNode {
    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);

        let frame_count = world.resource::<FrameCount>().0;
        warn!("NODE_UPDATE_START frame {}", frame_count);

        let Some(pipelines) = world.get_resource::<TerrainChunkPipelines>() else {
            warn!("No TerrainChunkPipelines yet");
            return;
        };
        let pipeline_cache = world.resource::<PipelineCache>();

        let Some(_voxel_vertex) =
            pipeline_cache.get_compute_pipeline(pipelines.compute_voxel_vertex_values_pipeline)
        else {
            warn!("Voxel vertex compute pipeline not ready yet.");
            return;
        };

        let Some(_cross_points) =
            pipeline_cache.get_compute_pipeline(pipelines.compute_voxel_cross_points_pipeline)
        else {
            if let CachedPipelineState::Err(e) =
                pipeline_cache.get_compute_pipeline_state(pipelines.compute_voxel_cross_points_pipeline)
            {
                error!("Failed to get voxel cross points: {:?}", e);
            }
            warn!("Voxel cross points pipeline not ready yet.");
            return;
        };

        let Some(_vertices) =
            pipeline_cache.get_compute_pipeline(pipelines.compute_vertices_pipeline)
        else {
            if let CachedPipelineState::Err(e) =
                pipeline_cache.get_compute_pipeline_state(pipelines.compute_vertices_pipeline)
            {
                error!("Failed to get mesh vertices: {:?}", e);
            }
            warn!("Mesh vertices pipeline not ready yet.");
            return;
        };

        let Some(_indices) =
            pipeline_cache.get_compute_pipeline(pipelines.compute_indices_pipeline)
        else {
            if let CachedPipelineState::Err(e) =
                pipeline_cache.get_compute_pipeline_state(pipelines.compute_indices_pipeline)
            {
                error!("Failed to get mesh indices: {:?}", e);
            }
            warn!("Mesh indices pipeline not ready yet.");
            return;
        };

        warn!("NODE_UPDATE_PIPELINES_READY frame {}", frame_count);

        self.to_compute_entities.clear();
        for (entity, state, coord, mut compute_state) in self.query.iter_mut(world) {
            warn!(
                "NODE_UPDATE: entity {:?} coord {:?} state {:?} compute {:?}",
                entity, coord, *state, *compute_state
            );
            if *state == TerrainChunkMeshingState::Meshing
                && *compute_state == TerrainChunkComputeState::Computing
            {
                self.to_compute_entities.push(entity);
                *compute_state = TerrainChunkComputeState::Sending;
                warn!("NODE_TO_COMPUTE: coord {:?}", coord);
            }
        }
    }

    fn run<'w>(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext<'w>,
        world: &'w World,
    ) -> Result<(), render_graph::NodeRunError> {
        if self.to_compute_entities.is_empty() {
            return Ok(());
        }

        let pipeline_cache = world.resource::<PipelineCache>();
        let pipelines = world.resource::<TerrainChunkPipelines>();
        let mesh_buffers = world.resource::<TerrainChunkMeshBuffers>();
        let main_bind_groups = world.resource::<TerrainChunkBindGroups>();
        let terrain_setting = world.resource::<TerrainSetting>();

        let voxel_vertex_pipeline = pipeline_cache
            .get_compute_pipeline(pipelines.compute_voxel_vertex_values_pipeline)
            .expect("Voxel vertex pipeline should be ready");
        let cross_points_pipeline = pipeline_cache
            .get_compute_pipeline(pipelines.compute_voxel_cross_points_pipeline)
            .expect("Cross points pipeline should be ready");
        let vertices_pipeline = pipeline_cache
            .get_compute_pipeline(pipelines.compute_vertices_pipeline)
            .expect("Vertices pipeline should be ready");
        let indices_pipeline = pipeline_cache
            .get_compute_pipeline(pipelines.compute_indices_pipeline)
            .expect("Indices pipeline should be ready");

        let command_encoder = render_context.command_encoder();
        let mut pass = command_encoder.begin_compute_pass(&ComputePassDescriptor {
            label: "terrain chunk mesh compute pass".into(),
            timestamp_writes: None,
        });

        for entity in self.to_compute_entities.iter() {
            if let Ok((entity, state, coord, _compute_state)) =
                self.query.get_manual(world, *entity)
            {
                assert!(*state == TerrainChunkMeshingState::Meshing);
                warn!("COMPUTE_STARTED: coord {:?}", coord);

                let voxel_num = terrain_setting.get_voxel_count_in_compute();
                let ws = 4u32;
                let edge_wg = ((voxel_num + 1) as f32 / ws as f32).ceil() as u32;
                let voxel_wg = (voxel_num as f32 / ws as f32).ceil() as u32;

                let offsets = mesh_buffers.get_buffers_dynamic_offset(entity);

                pass.set_bind_group(0,
                    main_bind_groups.main_mesh_bind_group.as_ref().expect("no main bind group"),
                    offsets.as_slice());
                pass.set_bind_group(1,
                    main_bind_groups.map_bind_group.as_ref().expect("no map bind group"),
                    &[]);

                pass.set_pipeline(voxel_vertex_pipeline);
                pass.dispatch_workgroups(edge_wg, edge_wg, edge_wg);

                pass.set_pipeline(cross_points_pipeline);
                pass.dispatch_workgroups(edge_wg, edge_wg, edge_wg);

                pass.set_pipeline(vertices_pipeline);
                pass.dispatch_workgroups(voxel_wg, voxel_wg, voxel_wg);

                pass.set_pipeline(indices_pipeline);
                pass.dispatch_workgroups(voxel_wg, voxel_wg, voxel_wg);
            }
        }

        Ok(())
    }
}
