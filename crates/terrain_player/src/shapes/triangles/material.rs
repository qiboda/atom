use bevy::render::mesh::PrimitiveTopology;
use bevy::render::render_resource::PolygonMode;
use bevy::{
    asset::Asset,
    prelude::{Color, Material, Mesh},
    reflect::TypePath,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, ShaderType},
};

use super::plugin::TRIANGLES_SHADER_HANDLE;

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct TriangleShaderSettings {
    pub color: Color,
}

impl Default for TriangleShaderSettings {
    fn default() -> Self {
        Self {
            color: Color::GREEN,
        }
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Copy, TypePath, Asset)]
#[uuid = "4d84d319-f6ab-470e-8005-7e76f5138842"]
pub struct TriangleMaterial {
    #[uniform(0)]
    pub settings: TriangleShaderSettings,
}

impl Default for TriangleMaterial {
    fn default() -> Self {
        Self {
            settings: TriangleShaderSettings::default(),
        }
    }
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
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        descriptor.primitive.polygon_mode = PolygonMode::Fill;
        descriptor.primitive.topology = PrimitiveTopology::TriangleList;

        let vertex_attributes = vec![Mesh::ATTRIBUTE_POSITION.at_shader_location(0)];

        let vertex_layout = layout.get_layout(&vertex_attributes)?;
        descriptor.vertex.buffers = vec![vertex_layout];

        Ok(())
    }
}
