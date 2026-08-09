use bevy::color::LinearRgba;
use bevy::mesh::PrimitiveTopology;
use bevy::render::render_resource::PolygonMode;
use bevy::{
    asset::Asset,
    prelude::{Material, Mesh},
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderType},
};

use super::plugin::TRIANGLES_SHADER_HANDLE;

/// 三角形渲染的 uniform 设置，随绑定组一并上传 GPU。
#[derive(Debug, Clone, Copy, ShaderType)]
pub struct TriangleShaderSettings {
    /// 三角形填充颜色。
    pub color: LinearRgba,
}

impl Default for TriangleShaderSettings {
    fn default() -> Self {
        Self {
            color: LinearRgba::GREEN,
        }
    }
}

/// 三角形材质：以 `TriangleList` 拓扑填充渲染，双面显示。
#[derive(AsBindGroup, Debug, Clone, Copy, TypePath, Asset, Default)]
pub struct TriangleMaterial {
    /// 材质 uniform 设置（填充颜色）。
    #[uniform(0)]
    pub settings: TriangleShaderSettings,
}

impl Material for TriangleMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        TRIANGLES_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> bevy::shader::ShaderRef {
        TRIANGLES_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        descriptor.primitive.polygon_mode = PolygonMode::Fill;
        descriptor.primitive.topology = PrimitiveTopology::TriangleList;

        let vertex_attributes = vec![Mesh::ATTRIBUTE_POSITION.at_shader_location(0)];

        let vertex_layout = layout.0.get_layout(&vertex_attributes)?;
        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::color::ColorToComponents;

    #[test]
    fn shader_settings_default() {
        let settings = TriangleShaderSettings::default();
        assert_eq!(settings.color.to_f32_array(), [0., 1., 0., 1.]);
    }

    #[test]
    fn material_default_uses_default_settings() {
        let material = TriangleMaterial::default();
        assert_eq!(
            material.settings.color.to_f32_array(),
            TriangleShaderSettings::default().color.to_f32_array()
        );
    }
}
