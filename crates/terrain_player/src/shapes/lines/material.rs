use bevy::{
    asset::Asset,
    prelude::{Color, Material, Mesh},
    reflect::TypePath,
    reflect::TypeUuid,
    render::render_resource::{AsBindGroup, PolygonMode, ShaderType},
};

use super::plugin::LINE_SHADER_HANDLE;

#[derive(Debug, Clone, Copy, ShaderType)]
pub struct LineShaderSettings {
    pub line_size: f32,
    pub color: Color,
}

impl Default for LineShaderSettings {
    fn default() -> Self {
        Self {
            line_size: 1.,
            color: Default::default(),
        }
    }
}

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Copy, TypePath, Asset, Default)]
#[uuid = "eba346dd-ab2b-474d-8fd5-502ab7e73a0a"]
#[bind_group_data(LineMaterialKey)]
pub struct LineMaterial {
    #[uniform(0)]
    pub settings: LineShaderSettings,
    pub use_vertex_color: bool,
}

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
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        LINE_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        LINE_SHADER_HANDLE.into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
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

        let vertex_layout = layout.get_layout(&vertex_attributes)?;
        descriptor.vertex.buffers = vec![vertex_layout];

        // descriptor.vertex.shader_defs = shader_defs.clone();

        // if let Some(fragment) = &mut descriptor.fragment {
        //     fragment.shader_defs = shader_defs;
        // }

        Ok(())
    }
}
