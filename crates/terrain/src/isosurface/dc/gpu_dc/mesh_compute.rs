use std::ops::Not;

// 相邻的lod都是同一级别的，可以直接overlay。
use bevy::{
    app::Plugin,
    prelude::*,
    render::{
        mesh::Indices,
        render_asset::RenderAssetUsages,
        render_graph::RenderGraph,
        render_resource::{Maintain, MapMode},
        renderer::{RenderDevice, RenderQueue},
        Render, RenderApp, RenderSet,
    },
    utils::HashMap,
};
use wgpu_types::{MaintainResult, PrimitiveTopology};

// To communicate between the main world and the render world we need a channel.
// Since the main world and render world run in parallel, there will always be a frame of latency
// between the data sent from the render world and the data received in the main world
//
// frame n => render world sends data through the channel at the end of the frame
// frame n + 1 => main world receives the data

use crossbeam_channel::{Receiver, Sender};

#[cfg(feature = "gpu_seam")]
use strum::IntoEnumIterator;

#[cfg(feature = "gpu_seam")]
use crate::isosurface::dc::gpu_dc::{
    bind_group_cache::{
        TerrainChunkSeamBindGroupCachedId, TerrainChunkSeamBindGroups,
        TerrainChunkSeamBindGroupsCache, TerrainChunkSeamBindGroupsCreateContext,
    },
    buffer_cache::{
        TerrainChunkSeamBufferCreateContext, TerrainChunkSeamBuffers, TerrainChunkSeamBuffersCache,
        TerrainChunkSeamKey,
    },
};

#[cfg(feature = "gpu_seam")]
use super::pipelines::TerrainChunkSeamComputeShadersPlugin;

use crate::{
    chunk_mgr::chunk::comp::{
        TerrainChunkAabb, TerrainChunkAddress, TerrainChunkBorderVertices, TerrainChunkSeamLod,
        TerrainChunkState,
    },
    isosurface::{
        csg::event::CSGOperationRecords,
        dc::gpu_dc::buffer_cache::{TerrainChunkMainBufferCreateContext, TerrainChunkMainBuffers},
    },
    materials::terrain_mat::MATERIAL_VERTEX_ATTRIBUTE,
    setting::TerrainSetting,
    tables::AxisType,
};

#[cfg(feature = "cpu_seam")]
use crate::isosurface::dc::gpu_dc::buffer_cache::TerrainChunkVertexInfo;
#[cfg(feature = "cpu_seam")]
use bevy::math::{bounding::Aabb3d, Vec3A};

use super::{
    bind_group_cache::{
        TerrainChunkMainBindGroupCachedId, TerrainChunkMainBindGroups,
        TerrainChunkMainBindGroupsCache, TerrainChunkMainBindGroupsCreateContext,
    },
    buffer_cache::{
        TerrainChunkMainBufferCachedId, TerrainChunkMainBuffersCache,
        TerrainChunkSeamBufferCachedId,
    },
    node::{TerrainChunkMeshComputeLabel, TerrainChunkMeshComputeNode},
    pipelines::{
        TerrainChunkDensityFieldComputeShadersPlugin, TerrainChunkMainComputeShadersPlugin,
        TerrainChunkPipelines, TerrainChunkVoxelComputeShadersPlugin,
    },
};

pub struct TerrainChunkGPUSeamMeshData {
    pub seam_mesh: Mesh,
    pub axis: AxisType,
}

pub struct TerrainChunkCPUSeamMeshData {
    pub seam_mesh: Mesh,
}

pub enum TerrainChunkSeamMeshData {
    GPUMesh(TerrainChunkGPUSeamMeshData),
    CPUMesh(TerrainChunkCPUSeamMeshData),
}

pub struct TerrainChunkMainMeshData {
    pub mesh: Mesh,
}

pub struct TerrainChunkMeshData {
    pub main_mesh_data: Option<TerrainChunkMainMeshData>,
    pub seam_mesh_data: Option<TerrainChunkSeamMeshData>,
    pub entity: Entity,
}

#[derive(Resource, Debug, Default)]
pub struct TerrainChunkRenderBorderVertices {
    pub map: HashMap<Entity, TerrainChunkBorderVertices>,
}

/// This will receive asynchronously any data sent from the render world
#[derive(Resource, Deref)]
pub struct TerrainChunkMeshDataMainWorldReceiver(Receiver<TerrainChunkMeshData>);

/// This will send asynchronously any data to the main world
#[derive(Resource, Deref)]
pub struct TerrainChunkMeshDataRenderWorldSender(Sender<TerrainChunkMeshData>);

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
        let (s, r) = crossbeam_channel::unbounded();
        app.insert_resource(TerrainChunkMeshDataMainWorldReceiver(r));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.insert_resource(TerrainChunkMeshDataRenderWorldSender(s));

        render_app.init_resource::<TerrainChunkPipelines>();
        render_app.init_resource::<TerrainChunkMainBindGroupsCache>();
        render_app.init_resource::<TerrainChunkMainBuffersCache>();
        render_app.init_resource::<TerrainChunkRenderBorderVertices>();

        #[cfg(feature = "gpu_seam")]
        render_app.init_resource::<TerrainChunkSeamBindGroupsCache>();
        #[cfg(feature = "gpu_seam")]
        render_app.init_resource::<TerrainChunkSeamBuffersCache>();

        render_app
            .add_systems(
                Render,
                (
                    prepare_main_buffers,
                    #[cfg(feature = "gpu_seam")]
                    prepare_seam_buffers,
                )
                    .in_set(RenderSet::PrepareResources),
            )
            .add_systems(
                Render,
                (
                    prepare_main_bind_group,
                    #[cfg(feature = "gpu_seam")]
                    prepare_seam_bind_group,
                )
                    .in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                (
                    map_and_read_buffer,
                    #[cfg(feature = "cpu_seam")]
                    crate::isosurface::dc::cpu_dc::seam_mesh::create_seam_mesh,
                    #[cfg(feature = "simulate_gpu_seam")]
                    super::simulate_gpu_seam::create_seam_mesh,
                )
                    .chain()
                    .after(RenderSet::Render)
                    .before(RenderSet::Cleanup),
            );

        let render_world = render_app.world_mut();
        let mesh_compute_node = TerrainChunkMeshComputeNode::from_world(render_world);

        let mut render_graph = render_world.resource_mut::<RenderGraph>();
        render_graph.add_node(TerrainChunkMeshComputeLabel, mesh_compute_node);
        // render_graph.add_node_edge(TerrainChunkMainMeshLabel, CameraDriverLabel);
    }

    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_plugins(TerrainChunkMainComputeShadersPlugin);
        render_app.add_plugins(TerrainChunkVoxelComputeShadersPlugin);
        render_app.add_plugins(TerrainChunkDensityFieldComputeShadersPlugin);
        #[cfg(feature = "gpu_seam")]
        render_app.add_plugins(TerrainChunkSeamComputeShadersPlugin);
    }
}

fn prepare_main_buffers(
    mut commands: Commands,
    query: Query<(
        Entity,
        &TerrainChunkAddress,
        &TerrainChunkAabb,
        &TerrainChunkState,
    )>,
    mut buffers_cache: ResMut<TerrainChunkMainBuffersCache>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    terrain_setting: Res<TerrainSetting>,
    csg_operation_records: Res<CSGOperationRecords>,
) {
    buffers_cache.reset_used_count();
    for (entity, address, aabb, state) in query.iter() {
        if state.contains(TerrainChunkState::CREATE_MAIN_MESH).not() {
            continue;
        }

        debug!("prepare main buffers: address: {:?}", address);

        let chunk_operations_data = csg_operation_records.get_chunk_gpu_data(address.0);

        let context = TerrainChunkMainBufferCreateContext {
            render_device: &render_device,
            render_queue: &render_queue,
            terrain_chunk_aabb: aabb.0,
            terrain_chunk_address: *address,
            terrain_setting: &terrain_setting,
            terrain_chunk_csg_operations: &chunk_operations_data,
        };

        let cached_id = match buffers_cache.acquire_terrain_chunk_buffers() {
            Some(buffer_cached_id) => buffer_cached_id,
            None => {
                let buffers = TerrainChunkMainBuffers::create_buffers(&context);
                buffers_cache.insert_terrain_chunk_buffers(buffers);
                buffers_cache.acquire_terrain_chunk_buffers().unwrap()
            }
        };

        let buffer_cached_id = TerrainChunkMainBufferCachedId(cached_id);

        let buffers = buffers_cache.get_buffers_mut(buffer_cached_id).unwrap();
        buffers.write_buffers_data(context);

        commands.entity(entity).insert(buffer_cached_id);
    }
}

#[cfg(feature = "gpu_seam")]
fn prepare_seam_buffers(
    mut commands: Commands,
    query: Query<(
        Entity,
        &TerrainChunkAddress,
        &TerrainChunkAabb,
        &TerrainChunkState,
        &TerrainChunkSeamLod,
    )>,
    mut buffers_cache: ResMut<TerrainChunkSeamBuffersCache>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    terrain_setting: Res<TerrainSetting>,
) {
    buffers_cache.reset_used_count();
    for (entity, address, aabb, state, seam_lod) in query.iter() {
        if state.contains(TerrainChunkState::CREATE_SEAM_MESH).not() {
            continue;
        }

        debug!("prepare seam buffers: address: {:?}", address);

        let terrain_chunk_seam_key = TerrainChunkSeamKey { lod: *seam_lod };

        let mut cached_ids = [0usize; 3];
        (0..3).for_each(|i| {
            let cached_id =
                match buffers_cache.acquire_terrain_chunk_buffers(terrain_chunk_seam_key) {
                    Some(buffer_cached_id) => buffer_cached_id,
                    None => {
                        let context = TerrainChunkSeamBufferCreateContext {
                            render_device: &render_device,
                            render_queue: &render_queue,
                            terrain_chunk_aabb: aabb.0,
                            terrain_chunk_address: *address,
                            terrain_chunk_seam_lod: *seam_lod,
                            terrain_setting: &terrain_setting,
                        };
                        let buffers = TerrainChunkSeamBuffers::create_buffers(context);
                        buffers_cache.insert_terrain_chunk_buffers(terrain_chunk_seam_key, buffers);
                        buffers_cache
                            .acquire_terrain_chunk_buffers(terrain_chunk_seam_key)
                            .unwrap()
                    }
                };
            cached_ids[i] = cached_id;
        });

        commands
            .entity(entity)
            .insert(TerrainChunkSeamBufferCachedId(cached_ids));
    }
}

#[allow(clippy::too_many_arguments)]
fn prepare_main_bind_group(
    mut commands: Commands,
    pipelines: Res<TerrainChunkPipelines>,
    render_device: Res<RenderDevice>,
    query: Query<(
        Entity,
        &TerrainChunkState,
        &TerrainChunkAabb,
        &TerrainChunkAddress,
        &TerrainChunkMainBufferCachedId,
    )>,
    buffers_cache: Res<TerrainChunkMainBuffersCache>,
    mut bind_groups_cache: ResMut<TerrainChunkMainBindGroupsCache>,
) {
    bind_groups_cache.reset_used_count();
    for (entity, state, aabb, address, buffer_cached_id) in query.iter() {
        if state.contains(TerrainChunkState::CREATE_MAIN_MESH).not() {
            continue;
        }

        debug!("prepare main bind groups: address: {:?}", address);

        let cached_id = match bind_groups_cache.acquire_terrain_chunk_bind_group() {
            Some(bind_groups_cached_id) => bind_groups_cached_id,
            None => {
                let buffers = buffers_cache.get_buffers(*buffer_cached_id).unwrap();
                let context = TerrainChunkMainBindGroupsCreateContext {
                    render_device: &render_device,
                    aabb,
                    pipelines: &pipelines,
                    buffers,
                };
                let bind_groups = TerrainChunkMainBindGroups::create_bind_groups(context);
                bind_groups_cache.insert(bind_groups);
                bind_groups_cache
                    .acquire_terrain_chunk_bind_group()
                    .unwrap()
            }
        };

        commands
            .entity(entity)
            .insert(TerrainChunkMainBindGroupCachedId(cached_id));
    }
}

#[cfg(feature = "gpu_seam")]
#[allow(clippy::too_many_arguments)]
fn prepare_seam_bind_group(
    mut commands: Commands,
    pipelines: Res<TerrainChunkPipelines>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    query: Query<(
        Entity,
        &TerrainChunkState,
        &TerrainChunkAabb,
        &TerrainChunkAddress,
        &TerrainChunkSeamLod,
        &TerrainChunkSeamBufferCachedId,
    )>,
    mut buffers_cache: ResMut<TerrainChunkSeamBuffersCache>,
    mut bind_groups_cache: ResMut<TerrainChunkSeamBindGroupsCache>,
    terrain_setting: Res<TerrainSetting>,
) {
    bind_groups_cache.reset_used_count();
    for (entity, state, aabb, address, seam_lod, buffer_cached_id) in query.iter() {
        if state.contains(TerrainChunkState::CREATE_SEAM_MESH).not() {
            continue;
        }

        debug!("prepare seam bind groups: address: {:?}", address);

        let terrain_chunk_seam_key = TerrainChunkSeamKey { lod: *seam_lod };

        let mut cached_ids = [0usize; 3];
        for i in 0..3 {
            let cached_id =
                match bind_groups_cache.acquire_terrain_chunk_bind_group(terrain_chunk_seam_key) {
                    Some(bind_groups_cached_id) => {
                        // 更新uniform buffer
                        let context = TerrainChunkSeamBufferCreateContext {
                            render_device: &render_device,
                            render_queue: &render_queue,
                            terrain_chunk_aabb: aabb.0,
                            terrain_chunk_seam_lod: *seam_lod,
                            terrain_chunk_address: *address,
                            terrain_setting: &terrain_setting,
                        };
                        let buffers = buffers_cache
                            .get_buffers_mut(terrain_chunk_seam_key, buffer_cached_id[i])
                            .unwrap();
                        buffers.update_buffers_reuse_info(context);

                        bind_groups_cached_id
                    }
                    None => {
                        let buffers = buffers_cache
                            .get_buffers(terrain_chunk_seam_key, buffer_cached_id[i])
                            .unwrap();
                        let context = TerrainChunkSeamBindGroupsCreateContext {
                            render_device: &render_device,
                            aabb,
                            pipelines: &pipelines,
                            buffers,
                        };
                        let bind_groups = TerrainChunkSeamBindGroups::create_bind_groups(context);
                        bind_groups_cache.insert(terrain_chunk_seam_key, bind_groups);
                        bind_groups_cache
                            .acquire_terrain_chunk_bind_group(terrain_chunk_seam_key)
                            .unwrap()
                    }
                };
            cached_ids[i] = cached_id;
        }

        commands
            .entity(entity)
            .insert(TerrainChunkSeamBindGroupCachedId(cached_ids));
    }
}

#[allow(clippy::type_complexity)]
fn map_and_read_buffer(
    render_device: Res<RenderDevice>,
    mut query: Query<
        (
            Entity,
            &TerrainChunkAddress,
            &TerrainChunkSeamLod,
            &TerrainChunkAabb,
            Option<&TerrainChunkMainBufferCachedId>,
            Option<&TerrainChunkSeamBufferCachedId>,
        ),
        Or<(
            With<TerrainChunkMainBufferCachedId>,
            With<TerrainChunkSeamBufferCachedId>,
        )>,
    >,
    main_buffers_cache: Res<TerrainChunkMainBuffersCache>,
    #[cfg(feature = "gpu_seam")] seam_buffers_cache: Res<TerrainChunkSeamBuffersCache>,
    sender: Res<TerrainChunkMeshDataRenderWorldSender>,
    terrain_setting: Res<TerrainSetting>,
    #[cfg(feature = "cpu_seam")] mut render_border_vertices: ResMut<
        TerrainChunkRenderBorderVertices,
    >,
) {
    let all_main_chunk_span = info_span!("all_main_chunk_map_async").entered();

    #[allow(unused_variables)]
    let voxel_num_in_chunk = terrain_setting.get_voxel_num_in_chunk();

    #[allow(unused_variables)]
    for (_, _, lod, _, main_buffers_id, seam_buffers_id) in query.iter() {
        if let Some(main_buffers_id) = main_buffers_id {
            let _one_main_chunk_span = info_span!("one_main_chunk_map_async").entered();

            let buffers = main_buffers_cache.get_buffers(*main_buffers_id).unwrap();

            let vertices_buffer_slice = buffers.mesh_vertices_buffer.get_staged_buffer().slice(..);
            // Maps the buffer so it can be read on the cpu
            vertices_buffer_slice.map_async(MapMode::Read, move |r| match r {
                // This will execute once the gpu is ready, so after the call to poll()
                Ok(_) => {}
                Err(err) => panic!("Failed to map vertex location buffer {err}"),
            });

            let indices_buffer_slice = buffers.mesh_indices_buffer.get_staged_buffer().slice(..);
            indices_buffer_slice.map_async(MapMode::Read, move |r| match r {
                Ok(_) => {}
                Err(err) => panic!("Failed to map indices buffer {err}"),
            });

            let mesh_vertices_num_buffer_slice = buffers
                .mesh_vertices_indices_count_buffer
                .get_staged_buffer()
                .slice(..);
            mesh_vertices_num_buffer_slice.map_async(MapMode::Read, move |r| match r {
                Ok(_) => {}
                Err(err) => panic!("Failed to map vertices num buffer {err}"),
            });
        }

        #[cfg(feature = "gpu_seam")]
        if let Some(seam_buffers_id) = seam_buffers_id {
            let terrain_chunk_seam_key = TerrainChunkSeamKey { lod: *lod };
            for i in 0..3 {
                let _one_seam_axis_lod_span = info_span!("one_seam_axis_lod_map_async").entered();
                let buffers = seam_buffers_cache
                    .get_buffers(terrain_chunk_seam_key, seam_buffers_id[i])
                    .unwrap();

                let vertices_buffer_slice = buffers
                    .seam_mesh_vertices_buffer
                    .get_staged_buffer()
                    .slice(..);
                // Maps the buffer so it can be read on the cpu
                vertices_buffer_slice.map_async(MapMode::Read, move |r| match r {
                    // This will execute once the gpu is ready, so after the call to poll()
                    Ok(_) => {}
                    Err(err) => panic!("Failed to map seam vertices buffer {err}"),
                });

                let indices_buffer_slice = buffers
                    .seam_mesh_indices_buffer
                    .get_staged_buffer()
                    .slice(..);
                indices_buffer_slice.map_async(MapMode::Read, move |r| match r {
                    Ok(_) => {}
                    Err(err) => panic!("Failed to map seam indices buffer {err}"),
                });

                let mesh_vertices_num_buffer_slice = buffers
                    .seam_mesh_vertices_indices_count_buffer
                    .get_staged_buffer()
                    .slice(..);
                mesh_vertices_num_buffer_slice.map_async(MapMode::Read, move |r| match r {
                    Ok(_) => {}
                    Err(err) => panic!("Failed to map vertices num buffer {err}"),
                });
            }
        }
    }

    drop(all_main_chunk_span);

    let main_chunk_poll_span = info_span!("main_chunk_render_device_poll").entered();

    match render_device.poll(Maintain::wait()) {
        MaintainResult::SubmissionQueueEmpty => {}
        MaintainResult::Ok => {
            panic!("MaintainResult should is SubmissionQueueEmpty!")
        }
    }

    drop(main_chunk_poll_span);

    let all_main_chunk_read_span = info_span!("all_main_chunk_read").entered();

    #[allow(unused_variables)]
    for (entity, address, lod, aabb, main_buffers_id, seam_buffers_id) in query.iter_mut() {
        if let Some(main_buffers_id) = main_buffers_id {
            let _one_main_chunk_read = info_span!("one_main_chunk_read").entered();
            let buffers = main_buffers_cache.get_buffers(*main_buffers_id).unwrap();

            let mesh_vertices_indices_count = buffers.mesh_vertices_indices_count_buffer.read();

            debug!(
                "main mesh vertices indices num: {:?}",
                mesh_vertices_indices_count
            );

            if mesh_vertices_indices_count.vertices_count > 0
                && mesh_vertices_indices_count.indices_count > 0
            {
                debug!("main map and read buffer: address: {:?}", address);

                let mut mesh = Mesh::new(
                    PrimitiveTopology::TriangleList,
                    RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
                );

                let vertices = buffers
                    .mesh_vertices_buffer
                    .read_size(mesh_vertices_indices_count.vertices_count as usize);
                let indices = buffers
                    .mesh_indices_buffer
                    .read_size(mesh_vertices_indices_count.indices_count as usize);

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
                        .map(|x| x.vertex_normal_materials.xyz())
                        .collect::<Vec<Vec3>>(),
                );
                mesh.insert_attribute(
                    MATERIAL_VERTEX_ATTRIBUTE,
                    vertices
                        .iter()
                        .map(|x| x.vertex_normal_materials.w as u32)
                        .collect::<Vec<u32>>(),
                );
                mesh.insert_indices(Indices::U32(indices));

                let main_mesh_data = TerrainChunkMainMeshData { mesh };
                match sender.send(TerrainChunkMeshData {
                    main_mesh_data: Some(main_mesh_data),
                    entity,
                    seam_mesh_data: None,
                }) {
                    Ok(_) => {}
                    Err(e) => error!("{}", e),
                }

                #[cfg(feature = "cpu_seam")]
                {
                    let chunk_min = aabb.0.min;
                    let level = address.0.depth();
                    let voxel_size = terrain_setting.get_voxel_size(level);
                    let mut border_vertices = TerrainChunkBorderVertices {
                        vertices: vertices
                            .into_iter()
                            .filter(|x| x.is_on_border(voxel_num_in_chunk as u32))
                            .collect::<Vec<TerrainChunkVertexInfo>>(),
                        ..Default::default()
                    };
                    border_vertices.vertices_aabb = border_vertices
                        .vertices
                        .iter()
                        .map(|x| {
                            let min = chunk_min
                                + Vec3A::new(
                                    x.vertex_local_coord.x as f32,
                                    x.vertex_local_coord.y as f32,
                                    x.vertex_local_coord.z as f32,
                                ) * voxel_size;
                            Aabb3d {
                                min,
                                max: min + Vec3A::splat(voxel_size),
                            }
                        })
                        .collect::<Vec<Aabb3d>>();

                    render_border_vertices.map.insert(entity, border_vertices);
                }
            }
            buffers.unmap();
        }

        #[cfg(feature = "gpu_seam")]
        if let Some(seam_buffers_id) = seam_buffers_id {
            let terrain_chunk_seam_key = TerrainChunkSeamKey { lod: *lod };
            for i in AxisType::iter() {
                let _one_seam_axis_lod_span = info_span!("one_seam_axis_lod_read").entered();
                let buffers = seam_buffers_cache
                    .get_buffers(terrain_chunk_seam_key, seam_buffers_id[i.to_index()])
                    .unwrap();

                let mesh_vertices_indices_count =
                    buffers.seam_mesh_vertices_indices_count_buffer.read();

                debug!(
                    "{:?} -> address: {:?}, seam mesh vertices indices num: {:?}",
                    i, address, mesh_vertices_indices_count,
                );

                if mesh_vertices_indices_count.vertices_count > 0
                    && mesh_vertices_indices_count.indices_count > 0
                {
                    let mut mesh = Mesh::new(
                        PrimitiveTopology::TriangleList,
                        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
                    );

                    debug!("seam map and read buffer: address: {:?}", address,);

                    let vertices = buffers
                        .seam_mesh_vertices_buffer
                        .read_size(mesh_vertices_indices_count.vertices_count as usize);
                    let indices = buffers
                        .seam_mesh_indices_buffer
                        .read_size(mesh_vertices_indices_count.indices_count as usize);

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
                            .map(|x| x.vertex_normal_materials.xyz())
                            .collect::<Vec<Vec3>>(),
                    );
                    mesh.insert_attribute(
                        MATERIAL_VERTEX_ATTRIBUTE,
                        vertices
                            .iter()
                            .map(|x| x.vertex_normal_materials.w as u32)
                            .collect::<Vec<u32>>(),
                    );
                    mesh.insert_indices(Indices::U32(indices));

                    let mesh_data = TerrainChunkGPUSeamMeshData {
                        seam_mesh: mesh,
                        axis: i,
                    };
                    match sender.send(TerrainChunkMeshData {
                        main_mesh_data: None,
                        entity,
                        seam_mesh_data: Some(TerrainChunkSeamMeshData::GPUMesh(mesh_data)),
                    }) {
                        Ok(_) => {}
                        Err(e) => error!("{}", e),
                    }
                }
                buffers.unmap();
            }
        }
    }

    drop(all_main_chunk_read_span);
}
