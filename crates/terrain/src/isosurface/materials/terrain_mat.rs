use bevy::pbr::{MaterialPipeline, MaterialPipelineKey};
use bevy::prelude::*;
use bevy::render::mesh::MeshVertexBufferLayoutRef;
use bevy::render::render_resource::{
    AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
};

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Hash)]
pub enum TerrainDebugType {
    Color,
    Normal,
}

#[derive(AsBindGroup, Clone, Default, Asset, Reflect)]
#[bind_group_data(TerrainMaterialKey)]
pub struct TerrainMaterial {
    #[uniform(0)]
    pub debug_color: LinearRgba,
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
}

#[derive(Clone, Debug, Reflect, PartialEq, Eq, Hash)]
pub struct TerrainMaterialKey {
    pub debug_type: Option<TerrainDebugType>,
}

impl From<&TerrainMaterial> for TerrainMaterialKey {
    fn from(value: &TerrainMaterial) -> Self {
        TerrainMaterialKey {
            debug_type: value.debug_type.clone(),
        }
    }
}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shader/terrain/terrain_material.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shader/terrain/terrain_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
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
        Ok(())
    }
}
