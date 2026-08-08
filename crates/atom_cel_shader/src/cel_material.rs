use bevy::{
    app::{App, Plugin},
    asset::Asset,
    color::LinearRgba,
    mesh::MeshVertexBufferLayoutRef,
    pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin},
    reflect::TypePath,
    render::render_resource::{
        AsBindGroup, Face, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

/// 赛璐璐基础材质插件：注册 [`CelMaterial`] 及其渲染管线。
#[derive(Debug, Default)]
pub struct CelMaterialPlugin;

impl Plugin for CelMaterialPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<CelMaterial>::default());
    }
}

/// 赛璐璐基础材质：平铺色卡通着色，背面剔除。
///
/// 顶点/片元 shader 使用内部 `cel_material.wgsl`，管线特化为背面剔除（`Face::Back`）。
#[derive(AsBindGroup, Debug, Default, TypePath, Clone, Asset)]
pub struct CelMaterial {
    /// 基础颜色（卡通平铺色）。
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
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = Some(Face::Back);
        Ok(())
    }
}
