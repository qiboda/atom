use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef};
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};
use wgpu::{Face, VertexFormat};

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Hash)]
pub enum TerrainDebugType {
    Color,
    Normal,
}

pub const MATERIAL_VERTEX_ATTRIBUTE: MeshVertexAttribute =
    MeshVertexAttribute::new("material", 100, VertexFormat::Uint32);

#[derive(AsBindGroup, Clone, Default, Asset, TypePath)]
#[bind_group_data(TerrainMaterialKey)]
pub struct TerrainMaterial {
    #[uniform(0)]
    pub lod: u32,
    pub debug_type: Option<TerrainDebugType>,

    #[texture(1)]
    #[sampler(2)]
    pub color_texture: Option<Handle<Image>>,

    #[texture(3)]
    #[sampler(4)]
    pub normal_texture: Option<Handle<Image>>,

    #[texture(5)]
    #[sampler(6)]
    pub metallic_texture: Option<Handle<Image>>,

    #[texture(7)]
    #[sampler(8)]
    pub roughness_texture: Option<Handle<Image>>,

    pub cull_mode: Option<Face>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TerrainMaterialKey {
    pub debug_type: Option<TerrainDebugType>,
    pub cull_mode: Option<Face>,
}

impl From<&TerrainMaterial> for TerrainMaterialKey {
    fn from(value: &TerrainMaterial) -> Self {
        TerrainMaterialKey {
            debug_type: value.debug_type,
            cull_mode: value.cull_mode,
        }
    }
}

impl Material for TerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/terrain/terrain_material.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/terrain/terrain_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;

        if let Some(debug_type) = key.bind_group_data.debug_type {
            match debug_type {
                TerrainDebugType::Color => {
                    let fragment = descriptor.fragment.as_mut().unwrap();
                    fragment.shader_defs.push("COLOR_DEBUG".into());
                }
                TerrainDebugType::Normal => {
                    let fragment = descriptor.fragment.as_mut().unwrap();
                    fragment.shader_defs.push("NORMAL_DEBUG".into());
                }
            }
        }
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            MATERIAL_VERTEX_ATTRIBUTE.at_shader_location(2),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}
