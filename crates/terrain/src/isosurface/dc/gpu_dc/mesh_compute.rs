use std::ops::Not;

// 相邻的lod都是同一级别的，可以直接overlay。
use bevy::{
    app::Plugin,
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues}, render_asset::{RenderAssetUsages, RenderAssets}, render_graph::RenderGraph, render_resource::Maintain, renderer::{RenderDevice, RenderQueue}, sync_world::MainEntity, texture::GpuImage, Render, RenderApp, RenderSet
    },
    utils::{HashMap, HashSet},
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
    chunk_mgr::chunk::comp::{
        TerrainChunkAabb, TerrainChunkAddress, TerrainChunkBorderVertices, TerrainChunkState,
    },
    isosurface::{
        csg::event::CSGOperationRecords,
        dc::gpu_dc::buffer_cache::{
            TerrainChunkMainBufferBindings, TerrainChunkMainDynamicBufferCreateContext,
        },
    },
    map::{
        compute_height::TerrainMapTextures,
        config::TerrainMapGpuConfig,
        topography::{MapFlatTerrainType, MapTerrainType},
    },
    materials::terrain_material::{BiomeColor, BIOME_VERTEX_ATTRIBUTE},
    setting::TerrainSetting,
};

use super::{
    bind_group_cache::{TerrainChunkMainBindGroups, TerrainChunkMainBindGroupsCreateContext},
    buffer_cache::{
        TerrainChunkMainBufferBindingsBuilder, TerrainChunkMainDynamicBufferReserveContext,
        TerrainChunkMainDynamicBuffers, TerrainChunkMainGlobalBufferCreateContext,
        TerrainChunkMainRecreateBindGroup,
    },
    node::{TerrainChunkMeshComputeLabel, TerrainChunkMeshComputeNode},
    pipelines::{
        TerrainChunkDensityFieldComputeShadersPlugin, TerrainChunkMainComputeShadersPlugin,
        TerrainChunkPipelines, TerrainChunkVoxelComputeShadersPlugin,
    },
};
use crate::isosurface::dc::gpu_dc::buffer_type::TerrainChunkVertexInfo;
use bevy::math::{bounding::Aabb3d, Vec3A};

pub struct TerrainChunkSeamMeshData {
    pub seam_mesh: Mesh,
}

impl TerrainChunkSeamMeshData {
    pub fn get_biomes(&self) -> [Option<BiomeColor>; MapFlatTerrainType::MAX] {
        get_biomes(&self.seam_mesh)
    }
}

fn get_biomes(mesh: &Mesh) -> [Option<BiomeColor>; MapFlatTerrainType::MAX] {
    let values = mesh.attribute(BIOME_VERTEX_ATTRIBUTE).unwrap();
    let mut set = HashSet::new();
    if let VertexAttributeValues::Uint32(biomes) = values {
        for biome in biomes.iter() {
            set.insert(*biome);
        }
    }
    let mut biomes = [None; MapFlatTerrainType::MAX];
    for (index, s) in set.iter().enumerate() {
        let biome = MapFlatTerrainType::from_repr(*s as usize).unwrap();
        let color = MapTerrainType::from(biome).get_color();
        let base_color = Vec4::new(
            color[0] as f32 / 255.0,
            color[1] as f32 / 255.0,
            color[2] as f32 / 255.0,
            color[3] as f32 / 255.0,
        );

        biomes[index] = Some(BiomeColor {
            biome: biome as u32,
            base_color,
        });
    }

    biomes
}

pub struct TerrainChunkMainMeshData {
    pub mesh: Mesh,
}

impl TerrainChunkMainMeshData {
    pub fn get_biomes(&self) -> [Option<BiomeColor>; MapFlatTerrainType::MAX] {
        get_biomes(&self.mesh)
    }
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

        let render_device = app.world().resource::<RenderDevice>();
        let main_dynamic_buffers = TerrainChunkMainDynamicBuffers::new(render_device);

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
        render_app.insert_resource(TerrainChunkMeshDataRenderWorldSender(s));

        render_app.init_resource::<TerrainChunkPipelines>();
        render_app.init_resource::<TerrainChunkMainBindGroups>();
        render_app.insert_resource(main_dynamic_buffers);
        render_app.init_resource::<TerrainChunkRenderBorderVertices>();

        render_app
            .add_systems(
                Render,
                (prepare_main_buffers,).in_set(RenderSet::PrepareResources),
            )
            .add_systems(
                Render,
                (prepare_main_bind_group,).in_set(RenderSet::PrepareBindGroups),
            )
            .add_systems(
                Render,
                (
                    map_and_read_buffer,
                    crate::isosurface::dc::cpu_dc::seam_mesh::create_seam_mesh,
                )
                    .chain()
                    .after(RenderSet::Render)
                    .before(RenderSet::Cleanup),
            )
            .add_systems(Render, clean_data_only_render.in_set(RenderSet::Cleanup));

        let render_world = render_app.world_mut();
        let mesh_compute_node = TerrainChunkMeshComputeNode::from_world(render_world);

        let mut render_graph = render_world.resource_mut::<RenderGraph>();
        render_graph.add_node(TerrainChunkMeshComputeLabel, mesh_compute_node);
    }

    fn build(&self, app: &mut App) {
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_plugins(TerrainChunkMainComputeShadersPlugin);
        render_app.add_plugins(TerrainChunkVoxelComputeShadersPlugin);
        render_app.add_plugins(TerrainChunkDensityFieldComputeShadersPlugin);
    }
}

fn prepare_main_buffers(
    query: Query<(
        Entity,
        &TerrainChunkAddress,
        &TerrainChunkAabb,
        &TerrainChunkState,
    )>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    terrain_setting: Res<TerrainSetting>,
    csg_operation_records: Res<CSGOperationRecords>,
    mut dynamic_buffers: ResMut<TerrainChunkMainDynamicBuffers>,
    map_gpu_config: Res<TerrainMapGpuConfig>,
) {
    dynamic_buffers.clear();

    dynamic_buffers.set_stride(&terrain_setting);

    let mut num = 0;
    let mut csg_operations_map = HashMap::new();

    let mut last_csg_operations_num = 0;
    for (entity, address, _aabb, state) in query.iter() {
        if state.contains(TerrainChunkState::CREATE_MAIN_MESH).not() {
            continue;
        }

        num += 1;

        let chunk_operations_data = csg_operation_records.get_chunk_gpu_data(address.0);
        let current_csg_operations_num =
            chunk_operations_data.as_ref().map(|x| x.len()).unwrap_or(1);
        csg_operations_map.insert(address, chunk_operations_data);

        let mut buffer_bindings = TerrainChunkMainBufferBindings::default();
        let builder = TerrainChunkMainBufferBindingsBuilder {
            current_index: num,
            last_csg_operations_num,
            current_csg_operations_num,
            terrain_setting: &terrain_setting,
            dynamic_buffers: &dynamic_buffers,
        };
        buffer_bindings.rebuild_binding_size(builder);
        dynamic_buffers.insert_terrain_chunk_buffer_bindings(entity, buffer_bindings);

        last_csg_operations_num += current_csg_operations_num;
    }

    if num == 0 {
        return;
    }

    let context = TerrainChunkMainDynamicBufferReserveContext {
        render_device: &render_device,
        render_queue: &render_queue,
        terrain_setting: &terrain_setting,
        instance_num: num,
        csg_operations_num: last_csg_operations_num,
    };

    dynamic_buffers.reserve_buffers(&context);

    for (entity, address, aabb, state) in query.iter() {
        if state.contains(TerrainChunkState::CREATE_MAIN_MESH).not() {
            continue;
        }

        let csg_data = csg_operations_map.get(address).unwrap();

        let context = TerrainChunkMainDynamicBufferCreateContext {
            terrain_chunk_aabb: **aabb,
            terrain_chunk_address: *address,
            terrain_setting: &terrain_setting,
            terrain_chunk_csg_operations: csg_data,
            entity,
        };
        dynamic_buffers.set_dynamic_buffers_data(context);
    }

    let context = TerrainChunkMainGlobalBufferCreateContext {
        terrain_map_gpu_config: &map_gpu_config,
    };
    dynamic_buffers.set_global_buffers(context);

    dynamic_buffers.write_dynamic_buffers(&render_device, &render_queue);
    dynamic_buffers.write_global_buffers(&render_device, &render_queue);
}

#[allow(clippy::too_many_arguments)]
fn prepare_main_bind_group(
    // mut commands: Commands,
    pipelines: Res<TerrainChunkPipelines>,
    render_device: Res<RenderDevice>,
    query: Query<(
        Entity,
        &TerrainChunkState,
        &TerrainChunkAabb,
        &TerrainChunkAddress,
    )>,
    mut bind_groups: ResMut<TerrainChunkMainBindGroups>,
    mut dynamic_buffers: ResMut<TerrainChunkMainDynamicBuffers>,
    height_map_images: Res<TerrainMapTextures>,
    images: Res<RenderAssets<GpuImage>>,
) {
    let mut num = 0;
    for (_entity, state, _aabb, _address) in query.iter() {
        if state.contains(TerrainChunkState::CREATE_MAIN_MESH).not() {
            continue;
        }

        num += 1;
    }

    if num == 0 {
        return;
    }

    let context = TerrainChunkMainBindGroupsCreateContext {
        render_device: &render_device,
        pipelines: &pipelines,
        dynamic_buffers: &dynamic_buffers,
        height_map_image: images.get(height_map_images.height_texture.id()).unwrap(),
        map_biome_image: images.get(height_map_images.biome_texture.id()).unwrap(),
    };
    bind_groups.create_bind_groups(context);

    dynamic_buffers.recreate_bind_group = TerrainChunkMainRecreateBindGroup::None;
}

#[allow(clippy::type_complexity)]
fn map_and_read_buffer(
    render_device: Res<RenderDevice>,
    mut query: Query<(
        Entity,
        &TerrainChunkState,
        &TerrainChunkAddress,
        &TerrainChunkAabb,
        &MainEntity,
    )>,
    main_buffers: Res<TerrainChunkMainDynamicBuffers>,
    sender: Res<TerrainChunkMeshDataRenderWorldSender>,
    terrain_setting: Res<TerrainSetting>,
    mut render_border_vertices: ResMut<TerrainChunkRenderBorderVertices>,
) {
    let mut num = 0;
    for (_entity, state, _address, _aabb, _main_entity) in query.iter() {
        if state.contains(TerrainChunkState::CREATE_MAIN_MESH) {
            num += 1;
        }
    }

    if num == 0 {
        return;
    }

    let all_main_chunk_span = info_span!("all_main_chunk_map_async").entered();

    let voxel_num_in_chunk = terrain_setting.get_voxel_num_in_chunk();

    main_buffers.map_async();

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

    for (entity, state, address, aabb, main_entity) in query.iter_mut() {
        if state.contains(TerrainChunkState::CREATE_MAIN_MESH) {
            let _one_main_chunk_read = info_span!("one_main_chunk_read").entered();

            let buffer_binding = main_buffers.get_buffer_bindings(entity).unwrap();
            let mesh_vertices_indices_count = main_buffers
                .mesh_vertices_indices_count_dynamic_buffer
                .read_one(
                    buffer_binding
                        .mesh_vertices_indices_count_buffer_binding
                        .offset,
                );

            debug!(
                "main mesh vertices indices num: {:?}, chunk_min: {}",
                mesh_vertices_indices_count, aabb.0.min,
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
                debug!("main map and read buffer: address: {:?}", address);

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
                mesh.insert_attribute(
                    BIOME_VERTEX_ATTRIBUTE,
                    vertices
                        .iter()
                        .map(|x| x.get_vertex_biome() as u32)
                        .collect::<Vec<u32>>(),
                );
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
                    seam_mesh_data: None,
                }) {
                    Ok(_) => {}
                    Err(e) => error!("{}", e),
                }
            }

            {
                if mesh_vertices_indices_count.vertices_count > 0 {
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

                    info!("main entity: {:?}, render entity: {:?}", main_entity.id(), entity);
                    // TODO 只有添加没有删除，会导致内存占用过大。
                    render_border_vertices.map.insert(main_entity.id(), border_vertices);
                }
            }
        }
    }

    main_buffers.unmap();

    drop(all_main_chunk_read_span);
}

fn clean_data_only_render(mut render_border_vertices: ResMut<TerrainChunkRenderBorderVertices>) {
    render_border_vertices.map.clear();
}
