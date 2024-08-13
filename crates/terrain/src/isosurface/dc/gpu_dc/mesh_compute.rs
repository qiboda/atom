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
};
use wgpu_types::{MaintainResult, PrimitiveTopology};

// To communicate between the main world and the render world we need a channel.
// Since the main world and render world run in parallel, there will always be a frame of latency
// between the data sent from the render world and the data received in the main world
//
// frame n => render world sends data through the channel at the end of the frame
// frame n + 1 => main world receives the data

use crossbeam_channel::{Receiver, Sender};

use crate::{
    chunk_mgr::chunk::{
        chunk_aabb::TerrainChunkAabb,
        state::{TerrainChunkAddress, TerrainChunkSeamLod, TerrainChunkState},
    },
    isosurface::{
        dc::{
            cpu_dc::cpu_seam::compute_seam_mesh,
            gpu_dc::{
                bind_group_cache::{
                    TerrainChunkSeamBindGroupCachedId, TerrainChunkSeamBindGroups,
                    TerrainChunkSeamBindGroupsCreateContext,
                },
                buffer_cache::{
                    TerrainChunkMainBufferCreateContext, TerrainChunkMainBuffers,
                    TerrainChunkSeamBufferCreateContext, TerrainChunkSeamBuffers,
                    TerrainChunkSeamKey,
                },
            },
        },
        materials::terrain_mat::MATERIAL_VERTEX_ATTRIBUTE,
    },
    setting::TerrainSetting,
    tables::SubNodeIndex,
};

use super::{
    bind_group_cache::{
        TerrainChunkMainBindGroupCachedId, TerrainChunkMainBindGroups,
        TerrainChunkMainBindGroupsCache, TerrainChunkMainBindGroupsCreateContext,
        TerrainChunkSeamBindGroupsCache,
    },
    buffer_cache::{
        TerrainChunkInfo, TerrainChunkMainBufferCachedId, TerrainChunkMainBuffersCache,
        TerrainChunkSeamBufferCachedId, TerrainChunkSeamBuffersCache,
    },
    node::{TerrainChunkMeshComputeLabel, TerrainChunkMeshComputeNode},
    pipelines::{TerrainChunkPipelines, TerrainChunkShadersPlugin},
};

pub struct TerrainChunkSeamMeshData {
    pub seam_mesh: Mesh,
    pub axis: usize,
}

pub struct TerrainChunkMeshData {
    pub main_mesh: Option<Mesh>,
    pub seam_mesh: Option<TerrainChunkSeamMeshData>,
    pub entity: Entity,
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
        render_app.init_resource::<TerrainChunkSeamBindGroupsCache>();
        render_app.init_resource::<TerrainChunkSeamBuffersCache>();

        render_app
            .add_systems(
                Render,
                (prepare_main_buffers, prepare_seam_buffers).in_set(RenderSet::PrepareResources),
            )
            .add_systems(
                Render,
                (prepare_main_bind_group, prepare_seam_bind_group)
                    .in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                (
                    map_and_read_buffer,
                    // create_seam_mesh,
                )
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
        render_app.add_plugins(TerrainChunkShadersPlugin);
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
) {
    buffers_cache.reset_used_count();
    for (entity, address, aabb, state) in query.iter() {
        if state.contains(TerrainChunkState::CREATE_MAIN_MESH).not() {
            continue;
        }

        debug!("prepare main buffers: address: {:?}", address);

        let cached_id = match buffers_cache.acquire_terrain_chunk_buffers() {
            Some(buffer_cached_id) => buffer_cached_id,
            None => {
                let context = TerrainChunkMainBufferCreateContext {
                    render_device: &render_device,
                    render_queue: &render_queue,
                    terrain_chunk_aabb: aabb.0,
                    terrain_chunk_address: *address,
                    terrain_setting: &terrain_setting,
                };
                let buffers = TerrainChunkMainBuffers::create_buffers(context);
                buffers_cache.insert_terrain_chunk_buffers(buffers);
                buffers_cache.acquire_terrain_chunk_buffers().unwrap()
            }
        };

        commands
            .entity(entity)
            .insert(TerrainChunkMainBufferCachedId(cached_id));
    }
}

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
    // return;

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
    render_queue: Res<RenderQueue>,
    query: Query<(
        Entity,
        &TerrainChunkState,
        &TerrainChunkAabb,
        &TerrainChunkAddress,
        &TerrainChunkMainBufferCachedId,
    )>,
    mut buffers_cache: ResMut<TerrainChunkMainBuffersCache>,
    mut bind_groups_cache: ResMut<TerrainChunkMainBindGroupsCache>,
    terrain_setting: Res<TerrainSetting>,
) {
    bind_groups_cache.reset_used_count();
    for (entity, state, aabb, address, buffer_cached_id) in query.iter() {
        if state.contains(TerrainChunkState::CREATE_MAIN_MESH).not() {
            continue;
        }

        debug!("prepare main bind groups: address: {:?}", address);

        let cached_id = match bind_groups_cache.acquire_terrain_chunk_bind_group() {
            Some(bind_groups_cached_id) => {
                // 更新uniform buffer
                let context = TerrainChunkMainBufferCreateContext {
                    render_device: &render_device,
                    render_queue: &render_queue,
                    terrain_chunk_aabb: aabb.0,
                    terrain_chunk_address: *address,
                    terrain_setting: &terrain_setting,
                };
                let buffers = buffers_cache.get_buffers_mut(*buffer_cached_id).unwrap();
                buffers.update_buffers_reuse_info(context);

                bind_groups_cached_id
            }
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
    // return;

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
    query: Query<
        (
            Entity,
            &TerrainChunkAddress,
            &TerrainChunkSeamLod,
            Option<&TerrainChunkMainBufferCachedId>,
            Option<&TerrainChunkSeamBufferCachedId>,
        ),
        Or<(
            With<TerrainChunkMainBufferCachedId>,
            With<TerrainChunkSeamBufferCachedId>,
        )>,
    >,
    main_buffers_cache: Res<TerrainChunkMainBuffersCache>,
    seam_buffers_cache: Res<TerrainChunkSeamBuffersCache>,
    sender: Res<TerrainChunkMeshDataRenderWorldSender>,
) {
    let all_main_chunk_span = info_span!("all_main_chunk_map_async").entered();

    for (_, _, lod, main_buffers_id, seam_buffers_id) in query.iter() {
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

    for (entity, address, lod, main_buffers_id, seam_buffers_id) in query.iter() {
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

                match sender.send(TerrainChunkMeshData {
                    main_mesh: Some(mesh),
                    entity,
                    seam_mesh: None,
                }) {
                    Ok(_) => {}
                    Err(e) => error!("{}", e),
                }
            }
            buffers.unmap();
        }

        if let Some(seam_buffers_id) = seam_buffers_id {
            let terrain_chunk_seam_key = TerrainChunkSeamKey { lod: *lod };
            for i in 0..3 {
                let _one_seam_axis_lod_span = info_span!("one_seam_axis_lod_read").entered();
                let buffers = seam_buffers_cache
                    .get_buffers(terrain_chunk_seam_key, seam_buffers_id[i])
                    .unwrap();

                let mesh_vertices_indices_count =
                    buffers.seam_mesh_vertices_indices_count_buffer.read();

                debug!(
                    "{} -> address: {:?}, seam mesh vertices indices num: {:?}",
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

                    match sender.send(TerrainChunkMeshData {
                        main_mesh: None,
                        entity,
                        seam_mesh: Some(TerrainChunkSeamMeshData {
                            seam_mesh: mesh,
                            axis: i,
                        }),
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

#[allow(dead_code)]
fn create_seam_mesh(
    query: Query<(
        Entity,
        &TerrainChunkState,
        &TerrainChunkAabb,
        &TerrainChunkAddress,
        &TerrainChunkSeamLod,
    )>,
    terrain_setting: Res<TerrainSetting>,
    sender: Res<TerrainChunkMeshDataRenderWorldSender>,
) {
    for (entity, state, aabb, address, seam_lod) in query.iter() {
        if state.contains(TerrainChunkState::CREATE_SEAM_MESH).not() {
            continue;
        }

        let chunk_min = aabb.min;
        let add_lod = seam_lod.get_lod(SubNodeIndex::X0Y0Z0);
        let level = address.0.level() + add_lod[0];
        let voxel_size = terrain_setting.get_voxel_size(level);
        let chunk_size = terrain_setting.get_chunk_size(level - add_lod[0]);
        let voxel_num = (chunk_size / voxel_size).round() as usize;

        let terrain_chunk_info = TerrainChunkInfo {
            chunk_min_location_size: Vec4::new(chunk_min.x, chunk_min.y, chunk_min.z, chunk_size),
            voxel_size,
            voxel_num: voxel_num as u32,
            qef_threshold: terrain_setting.qef_solver_threshold,
            qef_stddev: terrain_setting.qef_stddev,
        };

        let lod = seam_lod.to_uniform_buffer_array();
        let (mesh_x, mesh_y, mesh_z) = compute_seam_mesh(&terrain_chunk_info, lod);

        match sender.send(TerrainChunkMeshData {
            entity,
            main_mesh: None,
            seam_mesh: Some(TerrainChunkSeamMeshData {
                seam_mesh: mesh_x,
                axis: 0,
            }),
        }) {
            Ok(_) => {}
            Err(e) => error!("{}", e),
        }

        match sender.send(TerrainChunkMeshData {
            entity,
            main_mesh: None,
            seam_mesh: Some(TerrainChunkSeamMeshData {
                seam_mesh: mesh_y,
                axis: 1,
            }),
        }) {
            Ok(_) => {}
            Err(e) => error!("{}", e),
        }

        match sender.send(TerrainChunkMeshData {
            entity,
            main_mesh: None,
            seam_mesh: Some(TerrainChunkSeamMeshData {
                seam_mesh: mesh_z,
                axis: 2,
            }),
        }) {
            Ok(_) => {}
            Err(e) => error!("{}", e),
        }
    }
}
