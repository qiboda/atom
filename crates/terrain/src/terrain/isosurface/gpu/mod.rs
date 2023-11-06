// use bevy::{
//     prelude::*,
//     render::{
//         extract_resource::ExtractResourcePlugin,
//         render_graph::{Node, RenderGraph},
//         render_resource::{
//             BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
//             CachedComputePipelineId, PipelineCache, ShaderStages, StorageTextureAccess,
//             TextureFormat, TextureViewDimension,
//         },
//         renderer::RenderDevice,
//         Render, RenderApp, RenderSet,
//     },
// };
//
// #[derive(Default, Debug)]
// pub struct GPUIsosurfaceExtractionPlugin;
//
// impl Plugin for GPUIsosurfaceExtractionPlugin {
//     fn build(&self, app: &mut App) {
//         info!("add GPUIsosurfaceExtractionPlugin");
//
//         app.add_plugins(ExtractResourcePlugin::<RenderIsosurfaceExtraction>::default());
//         let render_app = app.sub_app_mut(RenderApp);
//         render_app.add_systems(Render, queue_bind_group.in_set(RenderSet::Queue));
//
//         let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
//         render_graph.add_node("isosurface_extraction", IsosurfaceExtractNode::default());
//         render_graph.add_node_edge(
//             "isosurface_extraction",
//             bevy::render::main_graph::node::CAMERA_DRIVER,
//         );
//     }
// }
//
// #[derive(Default, Debug)]
// pub struct RenderIsosurfaceExtraction;
//
// pub fn queue_bind_group() {}
//
// #[derive(Debug)]
// pub struct IsosurfaceExtractPipeline {
//     texture_bind_group_layout: BindGroupLayout,
//     init_pipeline: CachedComputePipelineId,
//     update_pipeline: CachedComputePipelineId,
// }
//
// impl FromWorld for IsosurfaceExtractPipeline {
//     fn from_world(world: &mut World) -> Self {
//         let texture_bind_group_layout =
//             world
//                 .resource::<RenderDevice>()
//                 .create_bind_group_layout(&BindGroupLayoutDescriptor {
//                     label: Some("IsosurfaceExtract"),
//                     entries: &[BindGroupLayoutEntry {
//                         binding: 0,
//                         visibility: ShaderStages::COMPUTE,
//                         ty: BindingType::StorageTexture {
//                             access: StorageTextureAccess::WriteOnly,
//                             format: TextureFormat::Rgba32Float,
//                             view_dimension: TextureViewDimension::D1,
//                         },
//                         count: None,
//                     }],
//                 });
//         // return IsosurfaceExtractPipeline {
//         //     texture_bind_group_layout,
//         //     init_pipeline: default(),
//         //     update_pipeline: default(),
//         // };
//         todo!()
//     }
// }
//
// #[derive(Default, Debug)]
// pub struct IsosurfaceExtractNode {}
//
// impl Node for IsosurfaceExtractNode {
//     fn update(&mut self, world: &mut World) {
//         let pipeline = world.resource();
//         let pipeline_cache = world.resource::<PipelineCache>();
//     }
// }
