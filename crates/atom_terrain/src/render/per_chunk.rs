//! Per-chunk indirect terrain rendering.
//!
//! Draws each ready chunk's vertex/index/indirect buffers via draw_indexed_indirect.

use bevy::{
    core_pipeline::schedule::{Core3d, Core3dSystems},
    mesh::VertexBufferLayout,
    prelude::*,
    render::{
        extract_resource::ExtractResourcePlugin,
        render_resource::{
            BindGroupLayoutDescriptor, BindGroupLayoutEntries,
            CachedRenderPipelineId, ColorTargetState, ColorWrites, CompareFunction,
            DepthBiasState, DepthStencilState, Face, FragmentState, FrontFace, IndexFormat,
            MultisampleState, PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology,
            RenderPassDescriptor, RenderPipelineDescriptor, ShaderStages, StencilState, StoreOp,
            TextureFormat, VertexAttribute, VertexFormat, VertexState, VertexStepMode,
            binding_types::uniform_buffer,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery, CurrentView},
        view::{ExtractedView, ViewDepthTexture, ViewTarget},
        RenderApp, RenderStartup,
    },
};

use crate::compute::chunk::ChunkManager;
use crate::compute::per_chunk::PerChunkComputePipeline;
use crate::axis_gizmo::GizmoCamera;
use crate::debug::TerrainDebugConfig;

#[derive(Resource)]
pub struct PerChunkRenderPipeline {
    solid_pipeline: CachedRenderPipelineId,
    wire_pipeline: CachedRenderPipelineId,
}

pub fn init_per_chunk_render(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
) {
    let entries = BindGroupLayoutEntries::sequential(
        ShaderStages::VERTEX,
        (uniform_buffer::<Mat4>(false),),
    );
    let bgl_desc = BindGroupLayoutDescriptor::new("pc_render_bgl", &entries);
    let _bgl = render_device.create_bind_group_layout("pc_render_bgl", &entries);

    let shader = asset_server.load("shaders/terrain/indirect_terrain.wgsl");
    let vertex_layout = VertexBufferLayout {
        array_stride: 32,
        step_mode: VertexStepMode::Vertex,
        attributes: vec![
            VertexAttribute { format: VertexFormat::Float32x3, offset: 0, shader_location: 0 },
            VertexAttribute { format: VertexFormat::Float32x3, offset: 16, shader_location: 1 },
        ],
    };

    let mk = |label: &'static str, polygon_mode, cull_mode, defs: Vec<&'static str>| {
        let shader_defs: Vec<_> = defs.into_iter().map(|s| s.into()).collect();
        pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some(label.into()),
            layout: vec![bgl_desc.clone()],
            immediate_size: 0,
            vertex: VertexState {
                shader: shader.clone(),
                shader_defs: shader_defs.clone(),
                entry_point: Some("vertex".into()),
                buffers: vec![vertex_layout.clone()],
            },
            fragment: Some(FragmentState {
                shader: shader.clone(),
                shader_defs,
                entry_point: Some("fragment".into()),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: Some(true),
                depth_compare: Some(CompareFunction::Greater),
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState { count: 4, mask: !0, alpha_to_coverage_enabled: false },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode,
                unclipped_depth: false,
                polygon_mode,
                conservative: false,
            },
            zero_initialize_workgroup_memory: false,
        })
    };

    let solid = mk("pc_render_solid", PolygonMode::Fill, Some(Face::Back), vec![]);
    let wire = mk("pc_render_wire", PolygonMode::Line, Some(Face::Back), vec!["WIREFRAME"]);
    commands.insert_resource(PerChunkRenderPipeline { solid_pipeline: solid, wire_pipeline: wire });
}

pub fn per_chunk_render_system(
    world: &World,
    view: ViewQuery<(&ExtractedView, &ViewTarget, &ViewDepthTexture)>,
    compute_pipeline: Res<PerChunkComputePipeline>,
    render_pipeline: Res<PerChunkRenderPipeline>,
    manager: Res<ChunkManager>,
    _debug_config: Res<TerrainDebugConfig>,
    pipeline_cache: Res<PipelineCache>,
    queue: Res<RenderQueue>,
    mut ctx: RenderContext,
) {
    // 跳过 gizmo 相机
    if world.get::<GizmoCamera>(world.resource::<CurrentView>().0).is_some() {
        return;
    }

    let (extracted_view, target, depth) = view.into_inner();
    let clip_from_world = extracted_view.clip_from_world.unwrap_or_else(|| {
        let view_from_world = extracted_view.world_from_view.affine().inverse();
        extracted_view.clip_from_view * view_from_world
    });

    let Some(solid_pso) = pipeline_cache.get_render_pipeline(render_pipeline.solid_pipeline) else {
        return;
    };

    let color_attachments = [Some(target.get_color_attachment())];
    let dsa = Some(depth.get_attachment(StoreOp::Store));
    let mut rp = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("pc_render"),
        color_attachments: &color_attachments,
        depth_stencil_attachment: dsa,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    rp.set_render_pipeline(solid_pso);

    for (&_cid, &slot_idx) in &manager.active {
        let Some(slot) = &compute_pipeline.slots[slot_idx] else { continue };
        if slot.pass < 6 { continue; }

        // 写 view-projection 到 uniform
        queue.write_buffer(
            &slot.uniform, 0,
            bytemuck::cast_slice(std::slice::from_ref(&clip_from_world)),
        );

        rp.set_bind_group(0, &slot.bg_render, &[]);
        rp.set_vertex_buffer(0, slot.vertices.slice(..));
        rp.set_index_buffer(slot.indices.slice(..), IndexFormat::Uint32);
        rp.draw_indexed_indirect(&slot.indirect, 0);
    }
}

/// Per-chunk rendering plugin
pub struct PerChunkRenderPlugin;

impl Plugin for PerChunkRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ExtractResourcePlugin::<TerrainDebugConfig>::default());
        let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return };
        render_app.add_systems(RenderStartup, init_per_chunk_render);
        render_app.add_systems(Core3d, per_chunk_render_system.in_set(Core3dSystems::MainPass));
    }
}
