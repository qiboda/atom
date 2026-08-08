use bevy::{
    asset::Asset,
    color::LinearRgba,
    prelude::{Material, Mesh},
    reflect::TypePath,
    render::render_resource::{AsBindGroup, PolygonMode, ShaderType},
};

use super::plugin::LINE_SHADER_HANDLE;

/// 线段渲染的 uniform 设置，随绑定组一并上传 GPU。
#[derive(Debug, Clone, Copy, ShaderType)]
pub struct LineShaderSettings {
    /// 线宽（像素，与 shader 中的 `line_size` 对齐）。
    pub line_size: f32,
    /// 线段基础颜色。
    pub color: LinearRgba,
}

impl Default for LineShaderSettings {
    fn default() -> Self {
        Self {
            line_size: 1.,
            color: LinearRgba::WHITE,
        }
    }
}

/// 线段材质：以 `LineList` 拓扑 + 线框模式渲染，支持顶点颜色。
#[derive(AsBindGroup, Debug, Clone, Copy, TypePath, Asset, Default)]
#[bind_group_data(LineMaterialKey)]
pub struct LineMaterial {
    /// 材质 uniform 设置（线宽与颜色）。
    #[uniform(0)]
    pub settings: LineShaderSettings,
    /// 是否启用顶点颜色（网格需带 `ATTRIBUTE_COLOR`）。
    pub use_vertex_color: bool,
}

/// 线段材质的管线特化键，用于区分不同的渲染管线变体。
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct LineMaterialKey {
    use_vertex_color: bool,
}

impl From<&LineMaterial> for LineMaterialKey {
    fn from(material: &LineMaterial) -> Self {
        LineMaterialKey {
            use_vertex_color: material.use_vertex_color,
        }
    }
}

impl Material for LineMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        LINE_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> bevy::shader::ShaderRef {
        LINE_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        descriptor.primitive.polygon_mode = PolygonMode::Line;

        // let mut shader_defs = vec![];
        let mut vertex_attributes = vec![Mesh::ATTRIBUTE_POSITION.at_shader_location(0)];

        // if key.bind_group_data.use_vertex_color && layout.contains(Mesh::ATTRIBUTE_COLOR) {
        //     shader_defs.push(ShaderDefVal::from("VERTEX_COLORS"));
        vertex_attributes.push(Mesh::ATTRIBUTE_COLOR.at_shader_location(1));
        // }

        let vertex_layout = layout.0.get_layout(&vertex_attributes)?;
        descriptor.vertex.buffers = vec![vertex_layout];

        // descriptor.vertex.shader_defs = shader_defs.clone();

        // if let Some(fragment) = &mut descriptor.fragment {
        //     fragment.shader_defs = shader_defs;
        // }

        Ok(())
    }
}
