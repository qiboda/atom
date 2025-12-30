// 相邻的lod都是同一级别的，可以直接overlay。
use bevy::{
    app::Plugin,
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    platform::collections::HashMap,
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        render_graph::RenderGraph,
        render_resource::PollType,
        renderer::{RenderDevice, RenderQueue},
        sync_world::MainEntity,
    },
};

use crate::{
    chunks::{
        chunk::{TerrainChunkCoord, TerrainChunkLod},
        mesh::{
            TerrainChunkMainMeshData, TerrainChunkMeshData, TerrainChunkMeshDataSender,
            TerrainChunkMeshingState,
            compute::{
                buffer::{
                    TerrainChunkMainBufferBindings, TerrainChunkMainDynamicBufferCreateContext,
                },
                data::TerrainChunkVertexInfo,
                node::{TerrainChunkMeshComputeLabel, TerrainChunkMeshComputeNode},
                pipelines::{
                    TerrainChunkDensityFieldComputeShadersPlugin,
                    TerrainChunkMeshComputeShadersPlugin,
                },
            },
        },
    },
    terrain::setting::TerrainSetting,
};

use super::{
    bind_group::{TerrainChunkBindGroups, TerrainChunkBindGroupsCreateContext},
    buffer::{
        TerrainChunkMainBufferBindingsBuilder, TerrainChunkMainDynamicBufferReserveContext,
        TerrainChunkMeshBuffers,
    },
    pipelines::{TerrainChunkPipelines, TerrainChunkVoxelComputeShadersPlugin},
};

/**
 * TODO 之后移到其他位置
 */
#[derive(Debug, Clone, Component, Default)]
pub struct TerrainChunkBorderVertices {
    pub vertices: Vec<TerrainChunkVertexInfo>,
}

#[derive(Resource, Debug, Default)]
pub struct TerrainChunkRenderBorderVertices {
    pub map: HashMap<Entity, TerrainChunkBorderVertices>,
}

bitflags::bitflags! {
    #[derive(PartialEq, Eq, Debug)]
    pub struct VoxelMaterial : u32 {
        const VoxelMaterialAir = 0x0;
        const VoxelMaterialBlock= 0x1;
    }
}

#[derive(Debug, Default)]
pub struct TerrainChunkMeshComputePlugin;

impl Plugin for TerrainChunkMeshComputePlugin {
    fn finish(&self, app: &mut App) {
        let render_device = app.world().resource::<RenderDevice>();
        let main_dynamic_buffers = TerrainChunkMeshBuffers::new(render_device);

        let max_buffer_size = render_device.limits().max_buffer_size;
        let max_storage_buffer_binding_size =
            render_device.limits().max_storage_buffer_binding_size;
        let max_uniform_buffer_binding_size =
            render_device.limits().max_uniform_buffer_binding_size;

        info!(
            "max_buffer_size: {:?}, max_storage size: {:?}, max_uniform size: {:?}",
            max_buffer_size, max_storage_buffer_binding_size, max_uniform_buffer_binding_size
        );

        let render_app = app.sub_app_mut(RenderApp);

        render_app.init_resource::<TerrainChunkPipelines>();
        render_app.init_resource::<TerrainChunkBindGroups>();
        render_app.insert_resource(main_dynamic_buffers);
        render_app.init_resource::<TerrainChunkRenderBorderVertices>();

        render_app
            .add_systems(
                Render,
                (prepare_main_buffers,).in_set(RenderSystems::PrepareResources),
            )
            .add_systems(
                Render,
                (prepare_main_bind_group,).in_set(RenderSystems::PrepareBindGroups),
            )
            .add_systems(
                Render,
                (
                    map_and_read_buffer,
                    // crate::isosurface::dc::cpu_dc::seam_mesh::create_seam_mesh,
                )
                    .chain()
                    .after(RenderSystems::Render)
                    .before(RenderSystems::Cleanup),
            )
            // .add_systems(
            //     Render,
            //     clean_data_only_render.in_set(RenderSystems::Cleanup),
            // )
            ;

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

fn prepare_main_buffers(
    query: Query<(
        Entity,
        &TerrainChunkCoord,
        &TerrainChunkLod,
        &TerrainChunkMeshingState,
    )>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    terrain_setting: Res<TerrainSetting>,
    mut dynamic_buffers: ResMut<TerrainChunkMeshBuffers>,
) {
    dynamic_buffers.clear();

    dynamic_buffers.set_stride(&terrain_setting);

    let mut num = 0;
    // let mut csg_operations_map = HashMap::new();

    for (entity, _coord, _aabb, state) in query.iter() {
        if *state != TerrainChunkMeshingState::Meshing {
            continue;
        }

        num += 1;

        let mut buffer_bindings = TerrainChunkMainBufferBindings::default();
        let builder = TerrainChunkMainBufferBindingsBuilder {
            current_index: num,
            terrain_setting: &terrain_setting,
            dynamic_buffers: &dynamic_buffers,
        };

        buffer_bindings.rebuild_binding_size(builder);
        dynamic_buffers.insert_terrain_chunk_buffer_bindings(entity, buffer_bindings);
    }

    if num == 0 {
        return;
    }

    let context = TerrainChunkMainDynamicBufferReserveContext {
        render_device: &render_device,
        render_queue: &render_queue,
        terrain_setting: &terrain_setting,
        instance_num: num,
    };

    dynamic_buffers.reserve_buffers(&context);

    for (entity, coord, lod, state) in query.iter() {
        if state != &TerrainChunkMeshingState::Meshing {
            continue;
        }

        let context = TerrainChunkMainDynamicBufferCreateContext {
            lod: lod.lod,
            terrain_chunk_coord: *coord,
            terrain_setting: &terrain_setting,
            entity,
        };
        dynamic_buffers.set_dynamic_buffers_data(context);
    }

    dynamic_buffers.write_dynamic_buffers(&render_device, &render_queue);
    // dynamic_buffers.write_global_buffers(&render_device, &render_queue);
}

#[allow(clippy::too_many_arguments)]
fn prepare_main_bind_group(
    pipelines: Res<TerrainChunkPipelines>,
    render_device: Res<RenderDevice>,
    query: Query<(
        Entity,
        &TerrainChunkMeshingState,
        &TerrainChunkLod,
        &TerrainChunkCoord,
    )>,
    mut bind_groups: ResMut<TerrainChunkBindGroups>,
    mut dynamic_buffers: ResMut<TerrainChunkMeshBuffers>,
) {
    let mut num = 0;
    for (_entity, state, _aabb, _address) in query.iter() {
        if *state != TerrainChunkMeshingState::Meshing {
            continue;
        }

        num += 1;
    }

    if num == 0 {
        return;
    }

    let context = TerrainChunkBindGroupsCreateContext {
        render_device: &render_device,
        pipelines: &pipelines,
        dynamic_buffers: &dynamic_buffers,
    };
    bind_groups.create_bind_groups(context);
    dynamic_buffers.should_recreate = false;
}

fn map_and_read_buffer(
    render_device: Res<RenderDevice>,
    mut query: Query<(
        Entity,
        &TerrainChunkMeshingState,
        &TerrainChunkLod,
        &TerrainChunkCoord,
        &MainEntity,
    )>,
    main_buffers: Res<TerrainChunkMeshBuffers>,
    sender: Res<TerrainChunkMeshDataSender>,
) {
    let mut num = 0;
    for (_entity, state, _lod, _coord, _main_entity) in query.iter() {
        if *state == TerrainChunkMeshingState::Meshing {
            num += 1;
        }
    }

    if num == 0 {
        return;
    }

    let all_main_chunk_span = info_span!("all_main_chunk_map_async").entered();

    // let voxel_count_in_chunk = terrain_setting.get_voxel_count_in_chunk();

    main_buffers.map_async();

    drop(all_main_chunk_span);

    let main_chunk_poll_span = info_span!("main_chunk_render_device_poll").entered();

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

    drop(main_chunk_poll_span);

    let all_main_chunk_read_span = info_span!("all_main_chunk_read").entered();

    for (entity, state, _lod, coord, main_entity) in query.iter_mut() {
        if *state == TerrainChunkMeshingState::Meshing {
            let _one_main_chunk_read = info_span!("one_main_chunk_read").entered();

            let buffer_binding = main_buffers
                .get_buffer_bindings(entity)
                .expect("Failed to get buffer bindings");
            let mesh_vertices_indices_count = main_buffers
                .mesh_vertices_indices_count_dynamic_buffer
                .read_one(
                    buffer_binding
                        .mesh_vertices_indices_count_buffer_binding
                        .offset,
                );

            debug!(
                "main mesh vertices indices num: {:?}, chunk_min: {:?}",
                mesh_vertices_indices_count, coord
            );

            let vertices = if mesh_vertices_indices_count.vertices_count > 0 {
                main_buffers
                    .mesh_vertices_dynamic_buffer
                    .read_inner_size::<TerrainChunkVertexInfo>(
                        buffer_binding.mesh_vertices_buffer_binding.offset,
                        mesh_vertices_indices_count.vertices_count as u64,
                    )
            } else {
                vec![]
            };

            let indices = if mesh_vertices_indices_count.indices_count > 0 {
                main_buffers
                    .mesh_indices_dynamic_buffer
                    .read_inner_size::<u32>(
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

                // mesh.compute_normals();

                // match mesh.generate_tangents() {
                //     Ok(_) => {}
                //     Err(e) => {
                //         warn!("generate_tangents error: {:?}", e);
                //         panic!("generate_tangents error: {:?}", e);
                //     }
                // }

                let main_mesh_data = TerrainChunkMainMeshData { mesh };
                match sender.send(TerrainChunkMeshData {
                    main_mesh_data: Some(main_mesh_data),
                    entity: main_entity.id(),
                }) {
                    Ok(_) => {}
                    Err(e) => error!("{}", e),
                }
            }

            // {
            //     if mesh_vertices_indices_count.vertices_count > 0 {
            //         let lod = lod.lod;
            //         let chunk_size = terrain_setting.get_chunk_size_by_lod(lod);
            //         let chunk_min = coord.to_world_pos(chunk_size);
            //         let voxel_size = terrain_setting.get_voxel_size_by_lod(lod);
            //         let mut border_vertices = TerrainChunkBorderVertices {
            //             vertices: vertices
            //                 .into_iter()
            //                 .filter(|x| x.is_on_border(voxel_count_in_chunk as u32))
            //                 .collect::<Vec<TerrainChunkVertexInfo>>(),
            //             ..Default::default()
            //         };
            //         border_vertices.vertices_aabb = border_vertices
            //             .vertices
            //             .iter()
            //             .map(|x| {
            //                 let min = chunk_min
            //                     + Vec3A::new(
            //                         x.vertex_local_coord.x as f32,
            //                         x.vertex_local_coord.y as f32,
            //                         x.vertex_local_coord.z as f32,
            //                     ) * voxel_size;
            //                 Aabb3d {
            //                     min,
            //                     max: min + Vec3A::splat(voxel_size),
            //                 }
            //             })
            //             .collect::<Vec<Aabb3d>>();

            //         info!(
            //             "main entity: {:?}, render entity: {:?}",
            //             main_entity.id(),
            //             entity
            //         );
            //         // TODO 只有添加没有删除，会导致内存占用过大。
            //         render_border_vertices
            //             .map
            //             .insert(main_entity.id(), border_vertices);
            //     }
            // }
        }
    }

    main_buffers.unmap();

    drop(all_main_chunk_read_span);
}

// fn clean_data_only_render(mut render_border_vertices: ResMut<TerrainChunkRenderBorderVertices>) {
//     render_border_vertices.map.clear();
// }
