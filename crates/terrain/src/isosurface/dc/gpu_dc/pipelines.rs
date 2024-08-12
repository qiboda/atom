use std::borrow::Cow;

use bevy::{
    prelude::*,
    render::{
        render_resource::{
            binding_types::{storage_buffer, storage_buffer_read_only, uniform_buffer},
            BindGroupLayout, BindGroupLayoutEntries, CachedComputePipelineId,
            ComputePipelineDescriptor, PipelineCache,
        },
        renderer::RenderDevice,
    },
};
use wgpu_types::ShaderStages;

use super::buffer_cache::{TerrainChunkInfo, TerrainChunkVertexInfo, VoxelEdgeCrossPoint};

const MAIN_COMPUTE_VERTICES_SHADER_ASSET_PATH: &str =
    "shaders/terrain/compute/main_mesh_compute_vertices.wgsl";
const MAIN_COMPUTE_INDICES_SHADER_ASSET_PATH: &str =
    "shaders/terrain/compute/main_mesh_compute_indices.wgsl";
const MAIN_COMPUTE_VOXEL_VERTEX_VALUES_SHADER_ASSET_PATH: &str =
    "shaders/terrain/compute/voxel_vertices.wgsl";
const MAIN_COMPUTE_VOXEL_CROSS_POINTS_SHADER_ASSET_PATH: &str =
    "shaders/terrain/compute/voxel_cross_points.wgsl";
const VOXEL_TYPE_SHADER_ASSET_PATH: &str = "shaders/terrain/compute/voxel_type.wgsl";
const VOXEL_UTILS_SHADER_ASSET_PATH: &str = "shaders/terrain/compute/voxel_utils.wgsl";
const DENSITY_FIELD_SHADER_ASSET_PATH: &str = "shaders/terrain/compute/density_field.wgsl";
const SEAM_COMPUTE_VERTICES_SHADER_ASSET_PATH: &str =
    "shaders/terrain/compute/seams/seam_mesh_compute_vertices.wgsl";
const SEAM_COMPUTE_INDICES_SHADER_ASSET_PATH: &str =
    "shaders/terrain/compute/seams/seam_mesh_compute_indices.wgsl";
const SEAM_UTILS_SHADER_ASSET_PATH: &str = "shaders/terrain/compute/seams/seam_mesh_utils.wgsl";
const SEAM_CLEAR_REUSED_BUFFERS_SHADER_ASSET_PATH: &str =
    "shaders/terrain/compute/seams/clear_reused_buffers.wgsl";

#[derive(Resource, Default)]
pub struct TerrainChunkShaders {
    pub main_compute_vertices_shader: Handle<Shader>,
    pub main_compute_indices_shader: Handle<Shader>,
    pub main_compute_voxel_cross_points_shader: Handle<Shader>,
    pub main_compute_voxel_vertex_values_shader: Handle<Shader>,

    pub density_field_shader: Handle<Shader>,
    pub voxel_type_shader: Handle<Shader>,
    pub voxel_utils_shader: Handle<Shader>,

    pub seam_compute_vertices_shader: Handle<Shader>,
    pub seam_compute_indices_shader: Handle<Shader>,
    pub seam_utils_shader: Handle<Shader>,
    pub seam_clear_reused_buffers_shader: Handle<Shader>,
}

// render world plugin
#[derive(Default)]
pub struct TerrainChunkShadersPlugin;

impl Plugin for TerrainChunkShadersPlugin {
    fn build(&self, app: &mut App) {
        // let render_app = app.get_sub_app_mut(RenderApp).unwrap();
        let world = app.world_mut();

        let shaders = TerrainChunkShaders {
            main_compute_vertices_shader: world.load_asset(MAIN_COMPUTE_VERTICES_SHADER_ASSET_PATH),
            main_compute_indices_shader: world.load_asset(MAIN_COMPUTE_INDICES_SHADER_ASSET_PATH),
            main_compute_voxel_cross_points_shader: world
                .load_asset(MAIN_COMPUTE_VOXEL_CROSS_POINTS_SHADER_ASSET_PATH),
            main_compute_voxel_vertex_values_shader: world
                .load_asset(MAIN_COMPUTE_VOXEL_VERTEX_VALUES_SHADER_ASSET_PATH),
            density_field_shader: world.load_asset(DENSITY_FIELD_SHADER_ASSET_PATH),
            voxel_type_shader: world.load_asset(VOXEL_TYPE_SHADER_ASSET_PATH),
            voxel_utils_shader: world.load_asset(VOXEL_UTILS_SHADER_ASSET_PATH),
            seam_compute_vertices_shader: world.load_asset(SEAM_COMPUTE_VERTICES_SHADER_ASSET_PATH),
            seam_compute_indices_shader: world.load_asset(SEAM_COMPUTE_INDICES_SHADER_ASSET_PATH),
            seam_utils_shader: world.load_asset(SEAM_UTILS_SHADER_ASSET_PATH),
            seam_clear_reused_buffers_shader: world
                .load_asset(SEAM_CLEAR_REUSED_BUFFERS_SHADER_ASSET_PATH),
        };
        app.insert_resource(shaders);
    }
}

#[derive(Resource)]
pub struct TerrainChunkPipelines {
    pub compute_voxel_vertex_values_bind_group_layout: BindGroupLayout,
    pub compute_voxel_vertex_values_pipeline: CachedComputePipelineId,

    pub compute_voxel_cross_points_bind_group_layout: BindGroupLayout,
    pub compute_voxel_cross_points_pipeline: CachedComputePipelineId,

    pub main_compute_vertices_bind_group_layout: BindGroupLayout,
    pub main_compute_vertices_pipeline: CachedComputePipelineId,

    pub main_compute_indices_bind_group_layout: BindGroupLayout,
    pub main_compute_indices_pipeline: CachedComputePipelineId,

    pub seam_compute_vertices_bind_group_layout: BindGroupLayout,
    pub seam_compute_vertices_x_axis_pipeline: CachedComputePipelineId,
    pub seam_compute_vertices_y_axis_pipeline: CachedComputePipelineId,
    pub seam_compute_vertices_z_axis_pipeline: CachedComputePipelineId,

    pub seam_compute_indices_bind_group_layout: BindGroupLayout,
    pub seam_compute_indices_x_axis_pipeline: CachedComputePipelineId,
    pub seam_compute_indices_y_axis_pipeline: CachedComputePipelineId,
    pub seam_compute_indices_z_axis_pipeline: CachedComputePipelineId,
    // pub seam_clear_reused_buffers_bind_group_layout: BindGroupLayout,
    // pub seam_clear_reused_buffers_pipeline: CachedComputePipelineId,
}

impl FromWorld for TerrainChunkPipelines {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let pipeline_cache = world.resource::<PipelineCache>();
        let shaders = world.resource::<TerrainChunkShaders>();

        // bind group layout

        let main_compute_vertices_bind_group_layout = render_device.create_bind_group_layout(
            "terrain chunk main mesh vertices bind group layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<TerrainChunkInfo>(false),
                    storage_buffer_read_only::<Vec<VoxelEdgeCrossPoint>>(false),
                    // vertex locations
                    storage_buffer::<Vec<TerrainChunkVertexInfo>>(false),
                    // vertex map
                    storage_buffer::<Vec<u32>>(false),
                    // vertices num
                    storage_buffer::<u32>(false),
                ),
            ),
        );

        let main_compute_indices_bind_group_layout = render_device.create_bind_group_layout(
            "terrain chunk main mesh indices bind group layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<TerrainChunkInfo>(false),
                    // voxel vertex values
                    storage_buffer_read_only::<Vec<f32>>(false),
                    // mesh vertex map
                    storage_buffer_read_only::<Vec<u32>>(false),
                    // mesh indices
                    storage_buffer::<Vec<u32>>(false),
                    // indices num
                    storage_buffer::<u32>(false),
                ),
            ),
        );

        let compute_voxel_cross_points_bind_group_layout = render_device.create_bind_group_layout(
            "terrain chunk voxel cross point bind group layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<TerrainChunkInfo>(false),
                    // voxel vertex values
                    storage_buffer_read_only::<Vec<f32>>(false),
                    // mesh indices
                    storage_buffer::<Vec<VoxelEdgeCrossPoint>>(false),
                ),
            ),
        );

        let compute_voxel_vertex_values_bind_group_layout = render_device.create_bind_group_layout(
            "terrain chunk voxel vertex values bind group layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<TerrainChunkInfo>(false),
                    // voxel vertex values
                    storage_buffer::<Vec<f32>>(false),
                ),
            ),
        );

        let seam_compute_vertices_bind_group_layout = render_device.create_bind_group_layout(
            "terrain chunk seam mesh vertices bind group layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<TerrainChunkInfo>(false),
                    uniform_buffer::<[UVec4; 16]>(false),
                    // vertex locations
                    storage_buffer::<Vec<TerrainChunkVertexInfo>>(false),
                    // vertex map
                    storage_buffer::<Vec<u32>>(false),
                    // vertices num
                    storage_buffer::<u32>(false),
                ),
            ),
        );

        let seam_compute_indices_bind_group_layout = render_device.create_bind_group_layout(
            "terrain chunk seam mesh indices bind group layout",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<TerrainChunkInfo>(false),
                    // mesh vertex map
                    storage_buffer_read_only::<Vec<u32>>(false),
                    // mesh indices
                    storage_buffer::<Vec<u32>>(false),
                    // indices num
                    storage_buffer::<u32>(false),
                ),
            ),
        );

        // let seam_clear_reused_buffers_bind_group_layout = render_device.create_bind_group_layout(
        //     "terrain chunk seam mesh indices bind group layout",
        //     &BindGroupLayoutEntries::sequential(
        //         ShaderStages::COMPUTE,
        //         (
        //             uniform_buffer::<TerrainChunkInfo>(false),
        //             // mesh vertex map
        //             storage_buffer::<Vec<u32>>(false),
        //         ),
        //     ),
        // );

        let main_compute_vertices_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("terrain chunk compute vertices pipeline".into()),
                layout: vec![main_compute_vertices_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shaders.main_compute_vertices_shader.clone_weak(),
                shader_defs: vec![],
                entry_point: Cow::from("compute_vertices"),
            });
        let main_compute_indices_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("terrain chunk compute indices pipeline".into()),
                layout: vec![main_compute_indices_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shaders.main_compute_indices_shader.clone_weak(),
                shader_defs: vec![],
                entry_point: Cow::from("compute_indices"),
            });

        let compute_voxel_vertex_values_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("terrain chunk voxel vertex values pipeline".into()),
                layout: vec![compute_voxel_vertex_values_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shaders.main_compute_voxel_vertex_values_shader.clone_weak(),
                shader_defs: vec![],
                entry_point: Cow::from("compute_voxel_vertices"),
            });

        let compute_voxel_cross_points_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("terrain chunk voxel cross points pipeline".into()),
                layout: vec![compute_voxel_cross_points_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shaders.main_compute_voxel_cross_points_shader.clone_weak(),
                shader_defs: vec![],
                entry_point: Cow::from("compute_voxel_cross_points"),
            });

        let seam_compute_vertices_x_axis_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("terrain chunk seam compute vertices pipeline".into()),
                layout: vec![seam_compute_vertices_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shaders.seam_compute_vertices_shader.clone_weak(),
                shader_defs: vec![],
                entry_point: Cow::from("compute_vertices_x_axis"),
            });
        let seam_compute_indices_x_axis_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("terrain chunk seam compute indices pipeline".into()),
                layout: vec![seam_compute_indices_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shaders.seam_compute_indices_shader.clone_weak(),
                shader_defs: vec![],
                entry_point: Cow::from("compute_indices_x_axis"),
            });
        let seam_compute_vertices_y_axis_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("terrain chunk seam compute vertices pipeline".into()),
                layout: vec![seam_compute_vertices_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shaders.seam_compute_vertices_shader.clone_weak(),
                shader_defs: vec![],
                entry_point: Cow::from("compute_vertices_y_axis"),
            });
        let seam_compute_indices_y_axis_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("terrain chunk seam compute indices pipeline".into()),
                layout: vec![seam_compute_indices_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shaders.seam_compute_indices_shader.clone_weak(),
                shader_defs: vec![],
                entry_point: Cow::from("compute_indices_y_axis"),
            });
        let seam_compute_vertices_z_axis_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("terrain chunk seam compute vertices pipeline".into()),
                layout: vec![seam_compute_vertices_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shaders.seam_compute_vertices_shader.clone_weak(),
                shader_defs: vec![],
                entry_point: Cow::from("compute_vertices_z_axis"),
            });
        let seam_compute_indices_z_axis_pipeline =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some("terrain chunk seam compute indices pipeline".into()),
                layout: vec![seam_compute_indices_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shaders.seam_compute_indices_shader.clone_weak(),
                shader_defs: vec![],
                entry_point: Cow::from("compute_indices_z_axis"),
            });
        // let seam_clear_reused_buffers_pipeline =
        //     pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
        //         label: Some("terrain chunk seam compute indices pipeline".into()),
        //         layout: vec![seam_clear_reused_buffers_bind_group_layout.clone()],
        //         push_constant_ranges: Vec::new(),
        //         shader: shaders.seam_clear_reused_buffers_shader.clone_weak(),
        //         shader_defs: vec![],
        //         entry_point: Cow::from("clear_reused_buffers"),
        //     });

        TerrainChunkPipelines {
            compute_voxel_vertex_values_bind_group_layout,
            compute_voxel_vertex_values_pipeline,
            compute_voxel_cross_points_bind_group_layout,
            compute_voxel_cross_points_pipeline,
            main_compute_vertices_bind_group_layout,
            main_compute_vertices_pipeline,
            main_compute_indices_bind_group_layout,
            main_compute_indices_pipeline,
            seam_compute_vertices_bind_group_layout,
            seam_compute_indices_bind_group_layout,
            seam_compute_vertices_x_axis_pipeline,
            seam_compute_indices_x_axis_pipeline,
            seam_compute_vertices_y_axis_pipeline,
            seam_compute_indices_y_axis_pipeline,
            seam_compute_vertices_z_axis_pipeline,
            seam_compute_indices_z_axis_pipeline,
            // seam_clear_reused_buffers_bind_group_layout,
            // seam_clear_reused_buffers_pipeline,
        }
    }
}
