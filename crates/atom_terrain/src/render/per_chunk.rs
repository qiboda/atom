//! Per-chunk GPU indirect terrain rendering.
//!
//! Draws each ready chunk's vertex/index/indirect buffers via draw_indexed_indirect.

use bevy::{
    core_pipeline::schedule::{Core3d, Core3dSystems},
    prelude::*,
    render::{
        extract_resource::ExtractResourcePlugin,
        render_resource::{
            BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntries,
            Buffer, BufferDescriptor, BufferUsages, CachedRenderPipelineId, ColorTargetState,
            ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Face, FragmentState,
            FrontFace, IndexFormat, MultisampleState, PipelineCache, PolygonMode, PrimitiveState,
            PrimitiveTopology, RenderPassDescriptor, RenderPipelineDescriptor, ShaderStages,
            StencilState, StoreOp, TextureFormat, VertexAttribute, VertexFormat, VertexState,
            VertexStepMode, binding_types::uniform_buffer,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery},
        view::{ExtractedView, ViewDepthTexture, ViewTarget},
        Render, RenderApp, RenderStartup,
    },
};

use crate::axis_gizmo::GizmoCamera;
use crate::compute::per_chunk::PerChunkComputePipeline;
use crate::compute::chunk::ChunkManager;
use crate::debug::TerrainDebugConfig;

#[derive(Resource)]
pub struct PerChunkRenderPipeline {
    solid: CachedRenderPipelineId,
    wire: CachedRenderPipelineId,
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
    let bgl = render_device.create_bind_group_layout("pc_render_bgl", &entries);

    let shader = asset_server.load("shaders/terrain/indirect_terrain.wgsl");
    let vertex_layout = VertexBufferLayout {
        array_stride: 32,
        step_mode: VertexStepMode::Vertex,
        attributes: vec![
            VertexAttribute { format: VertexFormat::Float32x3, offset: 0, shader_location: 0 },
            VertexAttribute { format: VertexFormat::Float32x3, offset: 16, shader_location: 1 },
        ],
    };

    let mk = |label: &str, polygon_mode, cull_mode, shader_defs: Vec<&str>| {
        let defs: Vec<_> = shader_defs.into_iter().map(|s: &str| s.into()).collect();
        pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some(label.into()),
            layout: vec![bgl_desc.clone()],
            immediate_size: 0,
            vertex: VertexState {
                shader: shader.clone(),
                shader_defs: defs.clone(),
                entry_point: Some("vertex".into()),
                buffers: vec![vertex_layout.clone()],
            },
            fragment: Some(FragmentState {
                shader: shader.clone(),
                shader_defs: defs,
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

    commands.insert_resource(PerChunkRenderPipeline { solid, wire });
}

pub fn per_chunk_render_system(
    world: &World,
    view: ViewQuery<(&ExtractedView, &ViewTarget, &ViewDepthTexture)>,
    compute_pipeline: Res<PerChunkComputePipeline>,
    render_pipeline: Res<PerChunkRenderPipeline>,
    manager: Res<ChunkManager>,
    debug_config: Res<TerrainDebugConfig>,
    pipeline_cache: Res<PipelineCache>,
    queue: Res<RenderQueue>,
    mut ctx: RenderContext,
) {
    // Skip gizmo camera
    if world.get::<GizmoCamera>(world.resource::<bevy::render::view::CurrentView>().0).is_some() {
        return;
    }

    let (extracted_view, target, depth) = view.into_inner();
    let clip_from_world = extracted_view.clip_from_world.unwrap_or_else(|| {
        let view_from_world = extracted_view.world_from_view.affine().inverse();
        extracted_view.clip_from_view * view_from_world
    });

    let solid_pipeline = match pipeline_cache.get_render_pipeline(render_pipeline.solid) {
        Some(p) => p,
        None => return,
    };

    let color_attachments = [Some(target.get_color_attachment())];
    let dsa = Some(depth.get_attachment(StoreOp::Store));
    let mut rp = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("per_chunk_render"),
        color_attachments: &color_attachments,
        depth_stencil_attachment: dsa,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    rp.set_render_pipeline(solid_pipeline);

    for (&_cid, &slot_idx) in &manager.active {
        let Some(slot) = &compute_pipeline.slots[slot_idx] else { continue };
        if slot.pass < 6 { continue; } // not ready

        // Write view-projection to the slot's uniform buffer (reuse compute uniform)
        // The uniform buffer is COPY_DST, so we can write the render uniform here.
        // We're reusing the same buffer slot for both compute and render uniforms.
        // This works because compute and render don't overlap in time.
        queue.write_buffer(
            &slot.uniform, 0,
            bytemuck::cast_slice(std::slice::from_ref(&clip_from_world)),
        );

        // For render, we need a bind group with only the uniform.
        // Since the compute bind group has 8 bindings but the render pipeline
        // expects only 1 binding (uniform), we need a separate bind group.
        // For now, create a simple binding on the fly.
        // Actually this won't work - the render pipeline expects a specific layout.
        // We need per-chunk render bind groups too.
    }
}
// FIXME: this module needs per-chunk render bind groups (separate from compute)
