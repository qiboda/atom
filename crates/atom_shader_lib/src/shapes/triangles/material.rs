use bevy::color::LinearRgba;
use bevy::render::mesh::{MeshVertexBufferLayoutRef, PrimitiveTopology};
use bevy::render::render_resource::PolygonMode;
use bevy::{
    asset::Asset,
    prelude::{Material, Mesh},
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderType},
};

use super::plugin::TRIANGLES_SHADER_HANDLE;

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct TriangleShaderSettings {
    pub color: LinearRgba,
}

impl Default for TriangleShaderSettings {
    fn default() -> Self {
        Self {
            color: LinearRgba::GREEN,
        }
    }
}

#[derive(AsBindGroup, Debug, Clone, Copy, TypePath, Asset, Default)]
pub struct TriangleMaterial {
    #[uniform(0)]
    pub settings: TriangleShaderSettings,
}

impl Material for TriangleMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        TRIANGLES_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        TRIANGLES_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
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
