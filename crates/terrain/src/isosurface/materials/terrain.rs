use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{AsBindGroup, AsBindGroupShaderType, ShaderRef, ShaderType};
use bevy::render::texture::GpuImage;
use bevy::{
    pbr::{ExtendedMaterial, MaterialExtension},
    prelude::*,
};

pub mod terrain_material {
    use bevy::reflect::Reflect;

    #[derive(Debug, Clone, Default, Reflect, PartialEq, Eq, Hash)]
    pub enum DebugType {
        #[default]
        Color,
        Normal,
    }
}

#[derive(AsBindGroup, Clone, Default, Asset, Reflect)]
#[bind_group_data(TerrainMaterialKey)]
#[uniform(30, TerrainMaterialUniform)]
pub struct TerrainMaterial {
    pub debug_type: Option<terrain_material::DebugType>,
    pub debug_color: Option<Color>,
}

#[derive(Clone, Debug, Reflect, PartialEq, Eq, Hash)]
pub struct TerrainMaterialKey {
    debug_type: Option<terrain_material::DebugType>,
}

impl From<&TerrainMaterial> for TerrainMaterialKey {
    fn from(value: &TerrainMaterial) -> Self {
        TerrainMaterialKey {
            debug_type: value.debug_type.clone(),
        }
    }
}

#[derive(Clone, Debug, ShaderType)]
pub struct TerrainMaterialUniform {
    pub debug_color: LinearRgba,
}

impl AsBindGroupShaderType<TerrainMaterialUniform> for TerrainMaterial {
    fn as_bind_group_shader_type(
        &self,
        _images: &RenderAssets<GpuImage>,
    ) -> TerrainMaterialUniform {
        TerrainMaterialUniform {
            debug_color: self.debug_color.unwrap_or(Color::WHITE).into(),
        }
    }
}

impl MaterialExtension for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shader/terrain/terrain_mat.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shader/terrain/terrain_mat.wgsl".into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialExtensionPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        _layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        key: bevy::pbr::MaterialExtensionKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        if let Some(debug_type) = key.bind_group_data.debug_type {
            match debug_type {
                terrain_material::DebugType::Color => {
                    let fragment = descriptor.fragment.as_mut().unwrap();
                    fragment.shader_defs.push("COLOR_DEBUG".into());
                }
                terrain_material::DebugType::Normal => {
                    let fragment = descriptor.fragment.as_mut().unwrap();
                    fragment.shader_defs.push("NORMAL_DEBUG".into());
                }
            }
        }
        Ok(())
    }
}

pub type TerrainExtendedMaterial = ExtendedMaterial<StandardMaterial, TerrainMaterial>;
