//! GPU indirect terrain rendering pipeline.
//!
//! Renders terrain mesh directly from `GlobalMeshPool` storage buffers via
//! `draw_indexed_indirect`, bypassing CPU readback for rendering.

use bevy::{
    core_pipeline::schedule::{Core3d, Core3dSystems},
    mesh::VertexBufferLayout,
    prelude::*,
    render::{
        render_resource::{
            binding_types::uniform_buffer, BindGroup, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntries, Buffer, BufferDescriptor,
            BufferUsages, CachedRenderPipelineId, ColorTargetState, ColorWrites, CompareFunction,
            DepthBiasState, DepthStencilState, Face, FragmentState, FrontFace, IndexFormat,
            MultisampleState, PipelineCache, PolygonMode, PrimitiveState, PrimitiveTopology,
            RenderPassDescriptor, RenderPipelineDescriptor, ShaderStages, StencilState, StoreOp,
            TextureFormat, VertexAttribute, VertexFormat, VertexState, VertexStepMode,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue, ViewQuery, CurrentView},
        view::{ExtractedView, ViewDepthTexture, ViewTarget},
        RenderApp, RenderStartup,
    },
};

use crate::axis_gizmo::GizmoCamera;
use crate::debug::TerrainDebugConfig;
use crate::compute::{
    global_compute::GlobalComputeState,
    global_pool::GlobalMeshPool,
};
/// Pipeline resource for GPU indirect terrain rendering.
///
/// Holds four pipeline variants (solid/wire × culled/double-sided),
/// selected at render time based on [`TerrainDebugConfig`].
#[allow(dead_code)]
#[derive(Resource)]
pub struct IndirectTerrainPipeline {
    /// Solid + back-face culling
    pub pipeline_solid_culled: CachedRenderPipelineId,
    /// Solid + double-sided
    pub pipeline_solid_double: CachedRenderPipelineId,
    /// Wireframe + back-face culling
    pub pipeline_wire_culled: CachedRenderPipelineId,
    /// Wireframe + double-sided
    pub pipeline_wire_double: CachedRenderPipelineId,
    /// Bind group layout descriptor (reused for pipeline creation).
    pub bgl_desc: BindGroupLayoutDescriptor,
    /// Bind group layout handle for bind group creation.
    pub bind_group_layout: BindGroupLayout,
    /// Dynamic uniform buffer holding the current view-projection matrix.
    /// Updated every frame via [`RenderQueue::write_buffer`].
    pub view_uniform_buffer: Buffer,
    /// Bind group referencing `view_uniform_buffer`.
    pub bind_group: BindGroup,
}
impl IndirectTerrainPipeline {}
/// Initialize the indirect terrain pipeline at render startup.
///
/// Creates the shader pipeline, uniform buffer, and bind group.
fn init_indirect_terrain_pipeline(
    mut commands: Commands,
    render_device: Res<RenderDevice>,
    pipeline_cache: Res<PipelineCache>,
    asset_server: Res<AssetServer>,
) {
    // ── Bind group layout: one uniform mat4 for clip_from_world ──
    let entries = BindGroupLayoutEntries::sequential(
        ShaderStages::VERTEX,
        (uniform_buffer::<Mat4>(false),),
    );
    let bgl_desc = BindGroupLayoutDescriptor::new("terrain_indirect_bgl", &entries);
    let bind_group_layout =
        render_device.create_bind_group_layout("terrain_indirect_bgl", &entries);

    // ── Uniform buffer (per-frame view-projection) ──
    let view_uniform_buffer = render_device.create_buffer(&BufferDescriptor {
        label: Some("terrain_indirect_view_uniform"),
        size: std::mem::size_of::<Mat4>() as u64,
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Bind group referencing the uniform buffer (identity initially).
    let initial_bg = render_device.create_bind_group(
        "terrain_indirect_bg",
        &bind_group_layout,
        &[BindGroupEntry {
            binding: 0,
            resource: view_uniform_buffer.as_entire_binding(),
        }],
    );

    // ── Load shader ──
    let shader = asset_server.load("shaders/terrain/indirect_terrain.wgsl");

    // ── Vertex layout matching TerrainChunkVertex (32 bytes, 2 attributes) ──
    //   position: Float32x3 at offset  0 (shader location 0)
    //   _pad0:    u32        at offset 12 (skipped)
    //   normal:   Float32x3  at offset 16 (shader location 1)
    //   _pad1:    u32        at offset 28 (skipped)
    //   stride = 32 bytes
    let vertex_layout = VertexBufferLayout {
        array_stride: 32,
        step_mode: VertexStepMode::Vertex,
        attributes: vec![
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            },
            VertexAttribute {
                format: VertexFormat::Float32x3,
                offset: 16,
                shader_location: 1,
            },
        ],
    };

    // ── Pipeline variants (solid/wire × culled/double-sided) ──
    let mk_pipeline = |label: &'static str,
                       polygon_mode: PolygonMode,
                       cull_mode: Option<Face>,
                       shader_defs: Vec<&'static str>,
                       depth_bias: DepthBiasState| {
        let defs: Vec<_> = shader_defs.into_iter().map(|d: &str| d.into()).collect();
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
                bias: depth_bias,
            }),
            multisample: MultisampleState {
                count: 4,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
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

    let no_bias = DepthBiasState::default();

    let solid_culled = mk_pipeline("terrain_indirect_solid_culled", PolygonMode::Fill, Some(Face::Back), vec![], no_bias);
    let solid_double = mk_pipeline("terrain_indirect_solid_double", PolygonMode::Fill, None, vec![], no_bias);
    let wire_culled  = mk_pipeline("terrain_indirect_wire_culled",  PolygonMode::Line, Some(Face::Back), vec!["WIREFRAME"], no_bias);
    let wire_double  = mk_pipeline("terrain_indirect_wire_double",  PolygonMode::Line, None, vec!["WIREFRAME"], no_bias);

    commands.insert_resource(IndirectTerrainPipeline {
        pipeline_solid_culled: solid_culled,
        pipeline_solid_double: solid_double,
        pipeline_wire_culled: wire_culled,
        pipeline_wire_double: wire_double,
        bgl_desc,
        bind_group_layout,
        view_uniform_buffer,
        bind_group: initial_bg,
    });
}

/// Render system for indirect terrain drawing.
///
/// Runs in the [`Core3d`](bevy::core_pipeline::schedule::Core3d) schedule after the
/// standard opaque 3D pass. Opens a new render pass (loading existing color/depth),
/// binds vertex/index buffers from [`GlobalMeshPool`], and issues
/// `draw_indexed_indirect` using the pool's indirect command buffer.
fn indirect_terrain_render_system(
    world: &World,
    view: ViewQuery<(&ExtractedView, &ViewTarget, &ViewDepthTexture)>,
    pipeline: Res<IndirectTerrainPipeline>,
    pool: Res<GlobalMeshPool>,
    state: Res<GlobalComputeState>,
    debug_config: Res<TerrainDebugConfig>,
    pipeline_cache: Res<PipelineCache>,
    queue: Res<RenderQueue>,
    mut ctx: RenderContext,
) {
    // 跳过 gizmo 相机 — 地形只渲染到主相机
    if world.get::<GizmoCamera>(world.resource::<CurrentView>().0).is_some() {
        return;
    }

    // Nothing to draw until the first rebuild completes.
    if !state.has_valid_data {
        return;
    }
    let (extracted_view, target, depth) = view.into_inner();

    // Build clip_from_world from the extracted view.
    let clip_from_world = extracted_view
        .clip_from_world
        .unwrap_or_else(|| {
            let view_from_world = extracted_view.world_from_view.affine().inverse();
            extracted_view.clip_from_view * view_from_world
        });

    // Write the view-projection matrix to the uniform buffer.
    queue.write_buffer(
        &pipeline.view_uniform_buffer,
        0,
        bytemuck::cast_slice(std::slice::from_ref(&clip_from_world)),
    );

    // Select solid pipeline variant based on double_sided.
    let solid_id = if debug_config.double_sided {
        pipeline.pipeline_solid_double
    } else {
        pipeline.pipeline_solid_culled
    };
    let Some(solid_pipeline) = pipeline_cache.get_render_pipeline(solid_id) else {
        return;
    };

    // Color attachment from the view target (load existing content).
    let color_attachments = [Some(target.get_color_attachment())];
    let depth_stencil_attachment = Some(depth.get_attachment(StoreOp::Store));

    let mut render_pass = ctx.begin_tracked_render_pass(RenderPassDescriptor {
        label: Some("terrain_indirect_pass"),
        color_attachments: &color_attachments,
        depth_stencil_attachment,
        timestamp_writes: None,
        occlusion_query_set: None,
        multiview_mask: None,
    });

    // ── First pass: solid rendering ──
    render_pass.set_render_pipeline(solid_pipeline);
    render_pass.set_bind_group(0, &pipeline.bind_group, &[]);
    render_pass.set_vertex_buffer(0, pool.vertices.slice(..));
    render_pass.set_index_buffer(pool.indices.slice(..), IndexFormat::Uint32);
    render_pass.draw_indexed_indirect(&pool.indirect, 0);

    // ── Second pass: green wireframe overlay (only when wireframe enabled) ──
    if debug_config.wireframe {
        let wire_id = if debug_config.double_sided {
            pipeline.pipeline_wire_double
        } else {
            pipeline.pipeline_wire_culled
        };
        if let Some(wire_pipeline) = pipeline_cache.get_render_pipeline(wire_id) {
            render_pass.set_render_pipeline(wire_pipeline);
            render_pass.set_bind_group(0, &pipeline.bind_group, &[]);
            render_pass.set_vertex_buffer(0, pool.vertices.slice(..));
            render_pass.set_index_buffer(pool.indices.slice(..), IndexFormat::Uint32);
            render_pass.draw_indexed_indirect(&pool.indirect, 0);
        }
    }
}

/// Plugin for GPU indirect terrain rendering.
///
/// Registers the pipeline initialization and the per-frame render system
/// in the `Core3d` schedule, after the standard opaque pass.
pub struct IndirectTerrainRenderPlugin;

impl Plugin for IndirectTerrainRenderPlugin {
    fn build(&self, app: &mut App) {
        // Extract TerrainDebugConfig from main world to render world
        app.add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<TerrainDebugConfig>::default());

        let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        // Initialize the render pipeline at startup (creates shaders, buffers, etc.)
        render_app.add_systems(RenderStartup, init_indirect_terrain_pipeline);
        // Render terrain in the main pass (loads existing color/depth via StoreOp::Load).
        render_app.add_systems(
            Core3d,
            indirect_terrain_render_system.in_set(Core3dSystems::MainPass),
        );
    }
}
