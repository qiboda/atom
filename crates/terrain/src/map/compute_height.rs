use atom_shader_lib::shaders_plugin;
use bevy::{
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::{RenderAssetUsages, RenderAssets},
        render_graph::{self, RenderGraph, RenderLabel},
        render_resource::{binding_types::texture_storage_2d, *},
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::GpuImage,
        Render, RenderApp, RenderSet,
    },
};
use binding_types::{sampler, texture_2d_array, uniform_buffer};
use std::borrow::Cow;

use crate::setting::TerrainSetting;

use super::TerrainMapImages;

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
            RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
        );

        image.texture_descriptor.usage = TextureUsages::COPY_DST
            | TextureUsages::STORAGE_BINDING
            | TextureUsages::TEXTURE_BINDING
            | TextureUsages::RENDER_ATTACHMENT;

        let image_handle = app.world_mut().add_asset(image);
        app.insert_resource(TerrainHeightMapImage::new(image_handle));
        app.add_plugins(ExtractResourcePlugin::<TerrainHeightMapImage>::default());

        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_systems(
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
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_plugins(TerrainHeightMapShadersPlugin);
        render_app.init_resource::<TerrainHeightMapPipeline>();
        render_app.init_resource::<TerrainHeightMapBuffer>();
        render_app.init_resource::<TerrainHeightMapBindGroups>();
    }
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct TerrainHeightMapLabel;

#[derive(Debug, Resource, Default, Clone, ExtractResource)]
pub struct TerrainHeightMapImage {
    pub texture: Handle<Image>,
}

impl TerrainHeightMapImage {
    pub fn new(texture: Handle<Image>) -> Self {
        Self { texture }
    }
}

#[derive(Resource, Default)]
struct TerrainHeightMapBindGroups(Option<BindGroup>);

fn prepare_bind_group(
    pipeline: Res<TerrainHeightMapPipeline>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    terrain_height_map_image: Res<TerrainHeightMapImage>,
    terrain_map_images: Res<TerrainMapImages>,
    render_device: Res<RenderDevice>,
    buffer: Res<TerrainHeightMapBuffer>,
    mut bind_group: ResMut<TerrainHeightMapBindGroups>,
) {
    if bind_group.0.is_none() {
        let height_image = gpu_images.get(&terrain_height_map_image.texture).unwrap();
        let biome_blend_images = gpu_images
            .get(&terrain_map_images.biome_blend_image)
            .unwrap();

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
) {
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
    computed: u32,
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
            computed: 0,
        }
    }
}

#[derive(Default)]
struct TerrainHeightComputeNode;

impl render_graph::Node for TerrainHeightComputeNode {
    fn update(&mut self, world: &mut World) {
        let mut pipeline = world.resource_mut::<TerrainHeightMapPipeline>();
        pipeline.computed += 1;
    }
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<TerrainHeightMapPipeline>();

        // if pipeline.computed < 5 {
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
            }
        // }
        Ok(())
    }
}
