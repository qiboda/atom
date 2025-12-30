use std::borrow::Cow;

use atom_shader_lib::shaders_plugin;
use bevy::{
    prelude::*,
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
            CachedComputePipelineId, ComputePipelineDescriptor, PipelineCache, SamplerBindingType,
            ShaderStages, TextureSampleType,
            binding_types::{
                sampler, storage_buffer, storage_buffer_read_only, texture_2d, uniform_buffer,
            },
        },
        renderer::RenderDevice,
    },
};

use super::data::{
    TerrainChunkCSGOperation, TerrainChunkInfo, TerrainChunkVertexInfo,
    TerrainChunkVerticesIndicesCount, VoxelEdgeCrossPoint,
};

shaders_plugin!(
    TerrainChunk,
    MeshCompute,
    (
        mesh_compute_vertices_shader -> "shaders/terrain/compute/main_mesh_compute_vertices.wgsl",
        mesh_compute_indices_shader -> "shaders/terrain/compute/main_mesh_compute_indices.wgsl",
        mesh_compute_voxel_cross_points_shader -> "shaders/terrain/compute/voxel_cross_points.wgsl",
        mesh_compute_voxel_vertex_values_shader -> "shaders/terrain/compute/voxel_vertices.wgsl",
        mesh_bind_group_shader -> "shaders/terrain/compute/main_mesh_bind_group.wgsl"
    )
);

shaders_plugin!(
    TerrainChunk,
    VoxelCompute,
    (
        voxel_type_shader -> "shaders/terrain/compute/voxel_type.wgsl",
        voxel_utils_shader -> "shaders/terrain/compute/voxel_utils.wgsl"
    )
);

shaders_plugin!(
    TerrainChunk,
    DensityFieldCompute,
    (
        density_field_shader -> "shaders/terrain/compute/density_field.wgsl",
        csg_type_shader -> "shaders/terrain/compute/csg/csg_type.wgsl",
        csg_utils_shader -> "shaders/terrain/compute/csg/csg_utils.wgsl"
    )
);

#[derive(Resource)]
pub struct TerrainChunkPipelines {
    pub compute_bind_group_layout: BindGroupLayout,
    pub compute_csg_bind_group_layout: BindGroupLayout,
    pub compute_map_bind_group_layout: BindGroupLayout,

    pub compute_voxel_vertex_values_pipeline: CachedComputePipelineId,
    pub compute_voxel_cross_points_pipeline: CachedComputePipelineId,
    pub compute_vertices_pipeline: CachedComputePipelineId,
    pub compute_indices_pipeline: CachedComputePipelineId,
}

impl FromWorld for TerrainChunkPipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let mesh_compute_shaders = world.resource::<TerrainChunkMeshComputeShaders>();

        let compute_bind_group_layout_desc = BindGroupLayoutDescriptor::new(
            "terrain chunk mesh vertices bind group layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<TerrainChunkInfo>(true),
                    // voxel vertex values
                    storage_buffer::<Vec<f32>>(true),
                    // voxel edge cross point
                    storage_buffer::<Vec<VoxelEdgeCrossPoint>>(true),
                    // mesh vertices
                    storage_buffer::<Vec<TerrainChunkVertexInfo>>(true),
                    // mesh indices
                    storage_buffer::<Vec<u32>>(true),
                    // vertex map
                    storage_buffer::<Vec<u32>>(true),
                    // vertices indices count
                    storage_buffer::<TerrainChunkVerticesIndicesCount>(true),
                ),
            ),
        );

        let compute_csg_bind_group_layout_desc = BindGroupLayoutDescriptor::new(
            "terrain chunk mesh csg bind group layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // csg info
                    uniform_buffer::<u32>(true),
                    // csg operation
                    storage_buffer_read_only::<Vec<TerrainChunkCSGOperation>>(true),
                ),
            ),
        );

        let compute_map_bind_group_layout_desc = BindGroupLayoutDescriptor::new(
            "terrain chunk mesh map bind group layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    // uniform_buffer::<TerrainSketchGpuConfig>(false),
                    // height map texture
                    texture_2d(TextureSampleType::Float { filterable: true }),
                    // height map sampler
                    sampler(SamplerBindingType::Filtering),
                    // biome map texture
                    texture_2d(TextureSampleType::Uint),
                    // biome map sampler
                    sampler(SamplerBindingType::NonFiltering),
                ),
            ),
        );

        // create pipeline

        let compute_vertices_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("terrain chunk compute vertices pipeline".into()),
                layout: vec![
                    compute_bind_group_layout_desc.clone(),
                    compute_csg_bind_group_layout_desc.clone(),
                    compute_map_bind_group_layout_desc.clone(),
                ],
                push_constant_ranges: Vec::new(),
                shader: mesh_compute_shaders.mesh_compute_vertices_shader.clone(),
                shader_defs: vec![],
                entry_point: Some(Cow::from("compute_vertices")),
                zero_initialize_workgroup_memory: false,
            });
        let main_compute_indices_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("terrain chunk compute indices pipeline".into()),
                layout: vec![
                    compute_bind_group_layout_desc.clone(),
                    compute_csg_bind_group_layout_desc.clone(),
                    compute_map_bind_group_layout_desc.clone(),
                ],
                push_constant_ranges: Vec::new(),
                shader: mesh_compute_shaders.mesh_compute_indices_shader.clone(),
                shader_defs: vec![],
                entry_point: Some(Cow::from("compute_indices")),
                zero_initialize_workgroup_memory: false,
            });

        let compute_voxel_vertex_values_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("terrain chunk voxel vertex values pipeline".into()),
                layout: vec![
                    compute_bind_group_layout_desc.clone(),
                    compute_csg_bind_group_layout_desc.clone(),
                    compute_map_bind_group_layout_desc.clone(),
                ],
                push_constant_ranges: Vec::new(),
                shader: mesh_compute_shaders
                    .mesh_compute_voxel_vertex_values_shader
                    .clone(),
                shader_defs: vec![],
                entry_point: Some(Cow::from("compute_voxel_vertices")),
                zero_initialize_workgroup_memory: false,
            });

        let compute_voxel_cross_points_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("terrain chunk voxel cross points pipeline".into()),
                layout: vec![
                    compute_bind_group_layout_desc.clone(),
                    compute_csg_bind_group_layout_desc.clone(),
                    compute_map_bind_group_layout_desc.clone(),
                ],
                push_constant_ranges: Vec::new(),
                shader: mesh_compute_shaders
                    .mesh_compute_voxel_cross_points_shader
                    .clone(),
                shader_defs: vec![],
                entry_point: Some(Cow::from("compute_voxel_cross_points")),
                zero_initialize_workgroup_memory: false,
            });

        TerrainChunkPipelines {
            compute_bind_group_layout: render_device.create_bind_group_layout(
                Some(compute_bind_group_layout_desc.label.as_ref()),
                compute_bind_group_layout_desc.entries.as_slice(),
            ),
            compute_csg_bind_group_layout: render_device.create_bind_group_layout(
                Some(compute_csg_bind_group_layout_desc.label.as_ref()),
                compute_csg_bind_group_layout_desc.entries.as_slice(),
            ),
            compute_map_bind_group_layout: render_device.create_bind_group_layout(
                Some(compute_map_bind_group_layout_desc.label.as_ref()),
                compute_map_bind_group_layout_desc.entries.as_slice(),
            ),
            compute_voxel_vertex_values_pipeline,
            compute_voxel_cross_points_pipeline,
            compute_vertices_pipeline,
            compute_indices_pipeline: main_compute_indices_pipeline,
        }
    }
}
