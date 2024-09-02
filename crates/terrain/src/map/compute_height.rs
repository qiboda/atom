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
use binding_types::{sampler, texture_2d_array, uniform_buffer};
use crossbeam_channel::{Receiver, Sender};
use std::{borrow::Cow, ops::Not};

use crate::{setting::TerrainSetting, TerrainState};

use super::TerrainInfoMap;

shaders_plugin!(
    Terrain,
    HeightMap,
    (
        height_map_shader -> "shaders/terrain/compute/height/height.wgsl",
        biome_shader -> "shaders/terrain/compute/biome.wgsl"
    )
);

#[derive(ShaderType, Default, Clone, Debug)]
pub struct TerrainHeightMapInfo {
    pub terrain_size: f32,
}

#[derive(Resource, Default)]
pub struct TerrainHeightMapBuffer {
    pub uniform_buffer: Option<UniformBuffer<TerrainHeightMapInfo>>,
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
        let mut image = Image::new_fill(
            Extent3d {
                width: 8192,
                height: 8192,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &[0, 0, 0, 255],
            TextureFormat::R32Float,
            RenderAssetUsages::RENDER_WORLD,
        );

        image.sampler = ImageSampler::linear();
        image.texture_descriptor.usage = TextureUsages::COPY_DST
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING;

        info!("TerrainHeightMapPlugin");
        let image_handle = app.world_mut().add_asset(image);

        app.add_plugins(ExtractResourcePlugin::<TerrainHeightMap>::default());
        app.insert_resource(TerrainHeightMap::new(image_handle));

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

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct TerrainHeightMapLabel;

#[derive(Debug, Resource, Default, Clone, ExtractResource)]
pub struct TerrainHeightMap {
    pub texture: Handle<Image>,
}

impl TerrainHeightMap {
    pub fn new(texture: Handle<Image>) -> Self {
        Self { texture }
    }
}

#[derive(Resource, Default)]
struct TerrainHeightMapBindGroups(Option<BindGroup>);

#[derive(Resource, Default)]
struct TerrainHeightMapEnable(bool);

#[allow(clippy::too_many_arguments)]
fn prepare_bind_group(
    pipeline: Res<TerrainHeightMapPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    terrain_height_map_image: Res<TerrainHeightMap>,
    terrain_map_images: Res<TerrainInfoMap>,
    render_device: Res<RenderDevice>,
    buffer: Res<TerrainHeightMapBuffer>,
    mut bind_group: ResMut<TerrainHeightMapBindGroups>,
    enable: Res<TerrainHeightMapEnable>,
) {
    if enable.0.not() {
        return;
    }

    if bind_group.0.is_none() {
        let height_image = gpu_images.get(&terrain_height_map_image.texture).unwrap();
        let biome_blend_images = gpu_images.get(&terrain_map_images.biome_blend_map).unwrap();

        bind_group.0 = Some(render_device.create_bind_group(
            None,
            &pipeline.texture_bind_group_layout,
            &BindGroupEntries::sequential((
                buffer.uniform_buffer.as_ref().unwrap().into_binding(),
                &biome_blend_images.texture_view,
                biome_blend_images.sampler.into_binding(),
                &height_image.texture_view,
            )),
        ));
    }
}

fn prepare_buffer(
    mut buffer: ResMut<TerrainHeightMapBuffer>,
    terrain_setting: Res<TerrainSetting>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    enable: Res<TerrainHeightMapEnable>,
) {
    if enable.0.not() {
        return;
    }

    if buffer.uniform_buffer.is_none() {
        let terrain_size = terrain_setting.get_terrain_size();
        info!("prepare buffer terrain size: {}", terrain_size);
        let mut height_map_info_uniform =
            UniformBuffer::from(TerrainHeightMapInfo { terrain_size });
        height_map_info_uniform.write_buffer(&render_device, &render_queue);
        buffer.uniform_buffer = Some(height_map_info_uniform);
    }
}

#[derive(Resource)]
struct TerrainHeightMapPipeline {
    texture_bind_group_layout: BindGroupLayout,
    update_pipeline: CachedComputePipelineId,
}

impl FromWorld for TerrainHeightMapPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let texture_bind_group_layout = render_device.create_bind_group_layout(
            "terrain height map",
            &BindGroupLayoutEntries::sequential(
                ShaderStages::COMPUTE,
                (
                    uniform_buffer::<TerrainHeightMapInfo>(false),
                    texture_2d_array(TextureSampleType::Float { filterable: true }),
                    sampler(SamplerBindingType::Filtering),
                    texture_storage_2d(TextureFormat::R32Float, StorageTextureAccess::WriteOnly),
                ),
            ),
        );

        let shaders: &TerrainHeightMapShaders = world.resource::<TerrainHeightMapShaders>();

        let pipeline_cache = world.resource::<PipelineCache>();
        let update_pipeline = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some(Cow::from("compute terrain height")),
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            shader: shaders.height_map_shader.clone(),
            shader_defs: vec![],
            entry_point: Cow::from("compute_terrain_height"),
        });

        TerrainHeightMapPipeline {
            texture_bind_group_layout,
            update_pipeline,
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

            if let Some(update_pipeline) =
                pipeline_cache.get_compute_pipeline(pipeline.update_pipeline)
            {
                let bind_group = world
                    .resource::<TerrainHeightMapBindGroups>()
                    .0
                    .as_ref()
                    .unwrap();

                let mut pass = render_context
                    .command_encoder()
                    .begin_compute_pass(&ComputePassDescriptor::default());

                pass.set_bind_group(0, bind_group, &[]);
                pass.set_pipeline(update_pipeline);
                pass.dispatch_workgroups(4096 / 16, 4096 / 16, 1);

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
