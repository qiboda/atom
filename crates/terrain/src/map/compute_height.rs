use atom_internal::app_state::AppState;
use atom_shader_lib::shaders_plugin;
use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::texture_storage_2d, *},
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::{GpuImage, ImageSampler},
        Extract, Render, RenderApp, RenderSet,
    },
};
use binding_types::{sampler, texture_2d_array, texture_storage_2d_array, uniform_buffer};
use crossbeam_channel::{Receiver, Sender};
use std::{borrow::Cow, ops::Not};

use crate::{setting::TerrainSetting, TerrainState};

use super::{config::TerrainMapSetting, TerrainInfoMap};

shaders_plugin!(
    Terrain,
    HeightMap,
    (
        height_map_shader -> "shaders/terrain/compute/height/height.wgsl",
        biome_filter_shader -> "shaders/terrain/compute/height/biome_filter.wgsl",
        map_type_shader -> "shaders/terrain/compute/height/map_type.wgsl",
        biome_shader -> "shaders/terrain/compute/biome.wgsl"
    )
);

#[derive(ShaderType, Default, Clone, Debug)]
pub struct TerrainMapInfo {
    pub terrain_size: f32,
    pub map_size: f32,
    pub pixel_num_per_kernel: u32,
    pub stride: u32,
}

#[derive(Resource, Default)]
pub struct TerrainHeightMapBuffer {
    pub uniform_buffer: Option<UniformBuffer<TerrainMapInfo>>,
}

#[derive(Default)]
pub struct TerrainHeightMapOver(bool);

/// This will receive asynchronously any data sent from the render world
#[derive(Resource, Deref)]
pub struct TerrainHeightMapMainWorldReceiver(Receiver<TerrainHeightMapOver>);

/// This will send asynchronously any data to the main world
#[derive(Resource, Deref)]
pub struct TerrainHeightMapRenderWorldSender(Sender<TerrainHeightMapOver>);

#[derive(Default)]
pub struct TerrainHeightMapPlugin;

impl Plugin for TerrainHeightMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::AppRunning), create_map_image);
        app.insert_resource(TerrainMapTextures::default());

        app.add_plugins(ExtractResourcePlugin::<TerrainMapTextures>::default());
        app.add_systems(PreUpdate, receive_msg_from_render_world);

        let render_app = app.sub_app_mut(RenderApp);
        render_app
            .add_systems(ExtractSchedule, extract_terrain_state)
            .add_systems(
                Render,
                (
                    prepare_buffer.in_set(RenderSet::PrepareResources),
                    prepare_bind_group.in_set(RenderSet::PrepareBindGroups),
                ),
            );

        let mut render_graph = render_app.world_mut().resource_mut::<RenderGraph>();
        render_graph.add_node(TerrainHeightMapLabel, TerrainHeightComputeNode);
        render_graph.add_node_edge(
            TerrainHeightMapLabel,
            bevy::render::graph::CameraDriverLabel,
        );
    }

    fn finish(&self, app: &mut App) {
        let (s, r) = crossbeam_channel::unbounded();
        app.insert_resource(TerrainHeightMapMainWorldReceiver(r));

        let render_app = app.sub_app_mut(RenderApp);
        render_app.insert_resource(TerrainHeightMapRenderWorldSender(s));

        render_app.add_plugins(TerrainHeightMapShadersPlugin);
        render_app.init_resource::<TerrainHeightMapPipeline>();
        render_app.init_resource::<TerrainHeightMapBuffer>();
        render_app.init_resource::<TerrainHeightMapBindGroups>();
        render_app.init_resource::<TerrainHeightMapEnable>();
    }
}

fn create_map_image(
    terrain_map_setting: Res<TerrainMapSetting>,
    mut map_textures: ResMut<TerrainMapTextures>,
    mut images: ResMut<Assets<Image>>,
) {
    let map_size = terrain_map_setting.get_map_size() as u32;

    let mut height_image = Image::new_fill(
        Extent3d {
            width: map_size,
            height: map_size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::R32Float,
        RenderAssetUsages::RENDER_WORLD,
    );

    height_image.sampler = ImageSampler::linear();
    height_image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    let mut biome_image = Image::new_fill(
        Extent3d {
            width: map_size,
            height: map_size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::R8Uint,
        RenderAssetUsages::RENDER_WORLD,
    );

    biome_image.sampler = ImageSampler::nearest();
    biome_image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    let height_image_handle = images.add(height_image);
    let biome_image_handle = images.add(biome_image);

    map_textures.height_texture = height_image_handle;
    map_textures.biome_texture = biome_image_handle;
    info!("create map image");
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct TerrainHeightMapLabel;

#[derive(Debug, Resource, Default, Clone, ExtractResource)]
pub struct TerrainMapTextures {
    pub height_texture: Handle<Image>,
    pub biome_texture: Handle<Image>,
}

impl TerrainMapTextures {
    pub fn new(height_texture: Handle<Image>, biome_texture: Handle<Image>) -> Self {
        Self {
            height_texture,
            biome_texture,
        }
    }
}

#[derive(Resource, Default)]
struct TerrainHeightMapBindGroups {
    pub height_bind_group: Option<BindGroup>,
    pub filter_bind_group: Option<BindGroup>,
}

#[derive(Resource, Default)]
struct TerrainHeightMapEnable(bool);

#[allow(clippy::too_many_arguments)]
fn prepare_bind_group(
    pipeline: Res<TerrainHeightMapPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    terrain_map_image: Res<TerrainMapTextures>,
    terrain_info_map_images: Res<TerrainInfoMap>,
    render_device: Res<RenderDevice>,
    buffer: Res<TerrainHeightMapBuffer>,
    mut bind_group: ResMut<TerrainHeightMapBindGroups>,
    enable: Res<TerrainHeightMapEnable>,
) {
    if enable.0.not() {
        return;
    }

    if bind_group.filter_bind_group.is_none() {
        let biome_image = gpu_images.get(&terrain_info_map_images.biome_map).unwrap();
        let biome_blend_image = gpu_images
            .get(&terrain_info_map_images.biome_blend_map)
            .unwrap();

        bind_group.filter_bind_group = Some(render_device.create_bind_group(
            None,
            &pipeline.filter_bind_group_layout,
            &BindGroupEntries::sequential((
                buffer.uniform_buffer.as_ref().unwrap().into_binding(),
                &biome_image.texture_view,
                &biome_blend_image.texture_view,
            )),
        ));
    }

    if bind_group.height_bind_group.is_none() {
        let height_image = gpu_images.get(&terrain_map_image.height_texture).unwrap();
        let biome_image = gpu_images.get(&terrain_map_image.biome_texture).unwrap();
        let biome_blend_image = gpu_images
            .get(&terrain_info_map_images.biome_blend_map)
            .unwrap();

        bind_group.height_bind_group = Some(render_device.create_bind_group(
            None,
            &pipeline.height_bind_group_layout,
            &BindGroupEntries::sequential((
                buffer.uniform_buffer.as_ref().unwrap().into_binding(),
                &biome_blend_image.texture_view,
                biome_blend_image.sampler.into_binding(),
                &height_image.texture_view,
                &biome_image.texture_view,
            )),
        ));
    }
}

fn prepare_buffer(
    mut buffer: ResMut<TerrainHeightMapBuffer>,
    terrain_setting: Res<TerrainSetting>,
    terrain_map_setting: Res<TerrainMapSetting>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    enable: Res<TerrainHeightMapEnable>,
) {
    if enable.0.not() {
        return;
    }

    if buffer.uniform_buffer.is_none() {
        let terrain_size = terrain_setting.get_terrain_size();
        let map_size = terrain_map_setting.get_map_size();
        info!(
            "prepare buffer terrain size: {}, map size: {}",
            terrain_size, map_size
        );

        let workgroup_size = (map_size / 16.0) as u32;
        let axis_pixel_num_per_kernel = if workgroup_size > 256 {
            workgroup_size / 256
        } else {
            1
        };
        let mut height_map_info_uniform = UniformBuffer::from(TerrainMapInfo {
            terrain_size,
            map_size,
            pixel_num_per_kernel: axis_pixel_num_per_kernel * axis_pixel_num_per_kernel,
            stride: axis_pixel_num_per_kernel,
        });
        height_map_info_uniform.write_buffer(&render_device, &render_queue);
        buffer.uniform_buffer = Some(height_map_info_uniform);
    }
}

#[derive(Resource)]
struct TerrainHeightMapPipeline {
    height_bind_group_layout: BindGroupLayout,
    filter_bind_group_layout: BindGroupLayout,

    compute_height_pipeline: CachedComputePipelineId,
    filter_pipeline: CachedComputePipelineId,
}

impl FromWorld for TerrainHeightMapPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let height_bind_group_layout = render_device.create_bind_group_layout(
            "terrain height map",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<TerrainMapInfo>(false),
                    texture_2d_array(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::WriteOnly),
                    texture_storage_2d(TextureFormat::R8Uint, StorageTextureAccess::WriteOnly),
                ),
            ),
        );

        let filter_bind_group_layout = render_device.create_bind_group_layout(
            "terrain biome filter",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<TerrainMapInfo>(false),
                    texture_2d_array(TextureSampleType::Float { filterable: false }),
                    texture_storage_2d_array(
                        TextureFormat::Rgba8Unorm,
                        StorageTextureAccess::WriteOnly,
                    ),
                ),
            ),
        );

        let shaders: &TerrainHeightMapShaders = world.resource::<TerrainHeightMapShaders>();

        let pipeline_cache = world.resource::<PipelineCache>();
        let compute_height_pipeline: CachedComputePipelineId = pipeline_cache
            .queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("compute terrain map height")),
                layout: vec![height_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shaders.height_map_shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::from("compute_terrain_map_height"),
            });

        let pipeline_cache = world.resource::<PipelineCache>();
        let filter_pipeline: CachedComputePipelineId =
            pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
                label: Some(Cow::from("filter biome")),
                layout: vec![filter_bind_group_layout.clone()],
                push_constant_ranges: Vec::new(),
                shader: shaders.biome_filter_shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::from("compute_terrain_map_biome"),
            });

        TerrainHeightMapPipeline {
            filter_bind_group_layout,
            height_bind_group_layout,
            compute_height_pipeline,
            filter_pipeline,
        }
    }
}

#[derive(Default)]
struct TerrainHeightComputeNode;

impl render_graph::Node for TerrainHeightComputeNode {
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let enable = world.resource::<TerrainHeightMapEnable>();

        if enable.0 {
            let pipeline_cache = world.resource::<PipelineCache>();
            let pipeline = world.resource::<TerrainHeightMapPipeline>();

            let map_setting = world.resource::<TerrainMapSetting>();
            let map_size = map_setting.get_map_size() as u32;
            let workgroup_size = 256.min(map_size / 16);

            let mut pass = render_context
                .command_encoder()
                .begin_compute_pass(&ComputePassDescriptor::default());

            info!("compute height workgroup size: {}", workgroup_size);

            if let Some(filter_pipeline) =
                pipeline_cache.get_compute_pipeline(pipeline.filter_pipeline)
            {
                let bind_group = world
                    .resource::<TerrainHeightMapBindGroups>()
                    .filter_bind_group
                    .as_ref()
                    .unwrap();

                pass.set_bind_group(0, bind_group, &[]);
                pass.set_pipeline(filter_pipeline);
                pass.dispatch_workgroups(workgroup_size, workgroup_size, 1);
            }

            if let Some(update_pipeline) =
                pipeline_cache.get_compute_pipeline(pipeline.compute_height_pipeline)
            {
                let bind_group = world
                    .resource::<TerrainHeightMapBindGroups>()
                    .height_bind_group
                    .as_ref()
                    .unwrap();

                pass.set_bind_group(0, bind_group, &[]);
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(workgroup_size, workgroup_size, 1);

                let sender = world.resource::<TerrainHeightMapRenderWorldSender>();

                if enable.0 {
                    match sender.send(TerrainHeightMapOver(true)) {
                        Ok(_) => {}
                        Err(_) => panic!("TerrainHeightMapRenderWorldSender send data fail!"),
                    }
                }
            }
        }
        Ok(())
    }
}

fn extract_terrain_state(
    mut enable: ResMut<TerrainHeightMapEnable>,
    state: Extract<Res<State<TerrainState>>>,
) {
    // TODO 会运行两次，需要解决。之后再处理。
    enable.0 = state.get() == &TerrainState::GenerateHeightMap;
    if enable.0 {
        info!("TerrainState::GenerateHeightMap enable: {}", enable.0);
    }
}

fn receive_msg_from_render_world(
    receiver: Res<TerrainHeightMapMainWorldReceiver>,
    mut state: ResMut<NextState<TerrainState>>,
) {
    if let Ok(data) = receiver.0.try_recv() {
        if data.0 {
            state.set(TerrainState::GenerateTerrainMesh);
        }
    }
}
