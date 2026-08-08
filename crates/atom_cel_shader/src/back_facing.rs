use atom_shader_lib::shaders_plugin;
use bevy::{
    mesh::MeshVertexBufferLayoutRef,
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    render::render_resource::{
        AsBindGroup, Face, RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
};

shaders_plugin!(
    CelShader,
    BackFacing,
    (
        back_facing -> "shaders/cel_shader/back_facing.wgsl"
    )
);

/// 描边（back-facing）材质插件：注册 [`BackFacingMaterial`] 与描边 shader。
#[derive(Debug, Default)]
pub struct BackFacingPlugin;
impl Plugin for BackFacingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<BackFacingMaterial>::default());
    }
}

/// 描边材质的组件：持有 [`BackFacingMaterial`] 资源句柄，挂到需要描边的实体上。
#[derive(Component, Debug, Reflect, Clone, Default)]
#[reflect(Component, Default)]
pub struct BackFacingMaterial3d(pub Handle<BackFacingMaterial>);

/// 描边材质：以「只渲染背面 + 膨胀顶点」的经典手法绘制卡通描边。
///
/// 管线特化为正面剔除（`Face::Front`），shader 将顶点沿法线方向略微外扩，
/// 实现环绕轮廓的描边效果。
#[derive(Default, AsBindGroup, TypePath, Debug, Clone, Asset)]
pub struct BackFacingMaterial {
    /// 描边颜色。
    #[uniform(0)]
    pub stroke_color: LinearRgba,
    /// 描边宽度。
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
        _pipeline: &MaterialPipeline,
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
