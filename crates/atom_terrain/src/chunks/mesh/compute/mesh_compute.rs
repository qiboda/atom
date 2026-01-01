use bevy::{
    app::Plugin,
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::{
        Render, RenderApp, RenderStartup, RenderSystems, render_graph::RenderGraph,
        render_resource::PollType, renderer::RenderDevice, sync_world::MainEntity,
    },
};

use crate::chunks::{
    chunk::TerrainChunkCoord,
    mesh::{
        channel::{TerrainChunkMeshData, TerrainChunkMeshDataSender},
        components::TerrainChunkMeshingState,
        compute::{
            bind_group::{TerrainChunkBindGroups, prepare_mesh_bind_group},
            buffer::{TerrainChunkMeshBuffers, prepare_mesh_buffers},
            node::{TerrainChunkMeshComputeLabel, TerrainChunkMeshComputeNode},
            pipelines::{
                TerrainChunkDensityFieldComputeShadersPlugin, TerrainChunkMeshComputeShadersPlugin,
                TerrainChunkVoxelComputeShadersPlugin, setup_terrain_chunk_pipelines,
            },
            types::TerrainChunkVertexInfo,
        },
    },
};

// bitflags::bitflags! {
//     #[derive(PartialEq, Eq, Debug)]
//     pub struct VoxelMaterial : u32 {
//         const VoxelMaterialAir = 0x0;
//         const VoxelMaterialBlock= 0x1;
//     }
// }

#[derive(Debug, Default)]
pub struct TerrainChunkMeshComputePlugin;

impl Plugin for TerrainChunkMeshComputePlugin {
    fn finish(&self, app: &mut App) {
        let render_device = app.world().resource::<RenderDevice>();

        let max_buffer_size = render_device.limits().max_buffer_size;
        let max_storage_buffer_binding_size =
            render_device.limits().max_storage_buffer_binding_size;
        let max_uniform_buffer_binding_size =
            render_device.limits().max_uniform_buffer_binding_size;

        info!(
            "max_buffer_size: {:?}, max_storage size: {:?}, max_uniform size: {:?}",
            max_buffer_size, max_storage_buffer_binding_size, max_uniform_buffer_binding_size
        );

        let mesh_dynamic_buffers = TerrainChunkMeshBuffers::new(render_device);

        let render_app = app.sub_app_mut(RenderApp);
        render_app.init_resource::<TerrainChunkBindGroups>();
        render_app.insert_resource(mesh_dynamic_buffers);

        render_app.add_systems(RenderStartup, setup_terrain_chunk_pipelines);

        render_app
            .add_systems(
                Render,
                (prepare_mesh_buffers,).in_set(RenderSystems::PrepareResources),
            )
            .add_systems(
                Render,
                (prepare_mesh_bind_group,).in_set(RenderSystems::PrepareBindGroups),
            )
            .add_systems(
                Render,
                (map_and_read_buffer,)
                    .chain()
                    .after(RenderSystems::Render)
                    .before(RenderSystems::Cleanup),
            );

        let render_world = render_app.world_mut();
        let mesh_compute_node = TerrainChunkMeshComputeNode::from_world(render_world);

        let mut render_graph = render_world.resource_mut::<RenderGraph>();
        render_graph.add_node(TerrainChunkMeshComputeLabel, mesh_compute_node);
    }

    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_plugins(TerrainChunkMeshComputeShadersPlugin);
        render_app.add_plugins(TerrainChunkVoxelComputeShadersPlugin);
        render_app.add_plugins(TerrainChunkDensityFieldComputeShadersPlugin);
    }
}

fn map_and_read_buffer(
    render_device: Res<RenderDevice>,
    mut query: Query<(
        Entity,
        &TerrainChunkMeshingState,
        &TerrainChunkCoord,
        &MainEntity,
    )>,
    mesh_buffers: Res<TerrainChunkMeshBuffers>,
    sender: Res<TerrainChunkMeshDataSender>,
) {
    let mut chunk_meshing_count = 0;
    for (_entity, state, _coord, _main_entity) in query.iter() {
        if *state == TerrainChunkMeshingState::Meshing {
            chunk_meshing_count += 1;
        }
    }

    if chunk_meshing_count == 0 {
        return;
    }

    {
        let _span = info_span!("all_mesh_chunk_map_async").entered();
        mesh_buffers.map_async();
    }

    {
        let _span = info_span!("mesh_chunk_render_device_poll").entered();
        match render_device.poll(PollType::wait_indefinitely()) {
            Ok(s) => match s {
                wgpu::PollStatus::QueueEmpty => {}
                _ => {
                    panic!("MaintainResult should is SubmissionQueueEmpty!")
                }
            },
            Err(err) => {
                error!("render_device poll error: {:?}", err);
            }
        }
    }

    let _span = info_span!("all_mesh_chunk_read").entered();

    for (entity, state, coord, main_entity) in query.iter_mut() {
        if *state != TerrainChunkMeshingState::Meshing {
            continue;
        }

        let _one_mesh_chunk_read_span = info_span!("one_mesh_chunk_read").entered();

        let buffer_binding = mesh_buffers
            .get_buffer_bindings(entity)
            .expect("Failed to get buffer bindings");
        let mesh_vertices_indices_count = mesh_buffers.mesh_vertices_indices_count_buffer.read_one(
            buffer_binding
                .mesh_vertices_indices_count_buffer_binding
                .offset,
        );

        trace!(
            "main mesh vertices indices num: {:?}, chunk_min: {:?}",
            mesh_vertices_indices_count, coord
        );

        let vertices = if mesh_vertices_indices_count.vertices_count > 0 {
            mesh_buffers
                .mesh_vertices_buffer
                .read_inner_size::<TerrainChunkVertexInfo>(
                    buffer_binding.mesh_vertices_buffer_binding.offset,
                    mesh_vertices_indices_count.vertices_count as u64,
                )
        } else {
            vec![]
        };

        let indices = if mesh_vertices_indices_count.indices_count > 0 {
            mesh_buffers.mesh_indices_buffer.read_inner_size::<u32>(
                buffer_binding.mesh_indices_buffer_binding.offset,
                mesh_vertices_indices_count.indices_count as u64,
            )
        } else {
            vec![]
        };

        if mesh_vertices_indices_count.vertices_count > 0
            && mesh_vertices_indices_count.indices_count > 0
        {
            debug!("main map and read buffer: coord: {:?}", coord);

            let mut mesh = Mesh::new(
                PrimitiveTopology::TriangleList,
                RenderAssetUsages::RENDER_WORLD,
            );

            mesh.insert_attribute(
                Mesh::ATTRIBUTE_POSITION,
                vertices
                    .iter()
                    .map(|x| x.vertex_location.xyz())
                    .collect::<Vec<Vec3>>(),
            );
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                vertices
                    .iter()
                    .map(|x| x.vertex_normal.xyz())
                    .collect::<Vec<Vec3>>(),
            );
            // mesh.insert_attribute(
            //     BIOME_VERTEX_ATTRIBUTE,
            //     vertices
            //         .iter()
            //         .map(|x| x.get_vertex_biome() as u32)
            //         .collect::<Vec<u32>>(),
            // );
            mesh.insert_indices(Indices::U32(indices));

            mesh.compute_normals();

            match mesh.generate_tangents() {
                Ok(_) => {}
                Err(e) => {
                    error!("generate_tangents error: {:?}", e);
                }
            }

            match sender.send(TerrainChunkMeshData {
                mesh,
                chunk_entity: main_entity.id(),
            }) {
                Ok(_) => {}
                Err(e) => error!("{}", e),
            }
        }
    }

    {
        let _span = info_span!("all_mesh_chunk_unmap").entered();
        mesh_buffers.unmap();
    }
}
