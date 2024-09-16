use bevy::{
    app::{App, Plugin},
    asset::Asset,
    color::LinearRgba,
    pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin},
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, Face, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
    },
};

#[derive(Debug, Default)]
pub struct CelMaterialPlugin;

impl Plugin for CelMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<CelMaterial>::default());
    }
}

#[derive(AsBindGroup, Debug, Default, TypePath, Clone, Asset)]
pub struct CelMaterial {
    #[uniform(0)]
    pub base_color: LinearRgba,
}

impl Material for CelMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/cel_shader/cel_material.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/cel_shader/cel_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = Some(Face::Back);
        Ok(())
    }
}
