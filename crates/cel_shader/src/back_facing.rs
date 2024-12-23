use atom_shader_lib::shaders_plugin;
use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, Face, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

shaders_plugin!(
    CelShader,
    BackFacing,
    (
        back_facing -> "shaders/cel_shader/back_facing.wgsl"
    )
);

#[derive(Debug, Default)]
pub struct BackFacingPlugin;
impl Plugin for BackFacingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<BackFacingMaterial>::default());
    }
}

#[derive(Component, Debug, Reflect, Clone, Default)]
#[reflect(Component, Default)]
pub struct BackFacingMaterial3d(pub Handle<BackFacingMaterial>);

#[derive(Default, AsBindGroup, TypePath, Debug, Clone, Asset)]
pub struct BackFacingMaterial {
    #[uniform(0)]
    pub stroke_color: LinearRgba,
    #[uniform(1)]
    pub stroke_width: f32,
}

impl Material for BackFacingMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/cel_shader/back_facing.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/cel_shader/back_facing.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = Some(Face::Front);
        // if let Some(depth_stencil) = descriptor.depth_stencil.as_mut() {
        //     depth_stencil.bias.slope_scale = 1.0;
        // }
        Ok(())
    }
}
