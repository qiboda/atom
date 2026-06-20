use std::ops::Not;

use bevy::mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef};
use bevy::pbr::{MaterialPipeline, MaterialPipelineKey, MeshPipelineKey, StandardMaterialFlags};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssets;
use bevy::render::render_resource::{
    AsBindGroup, AsBindGroupShaderType, Face, RenderPipelineDescriptor, ShaderType,
    SpecializedMeshPipelineError, TextureFormat,
};
use bevy::render::texture::GpuImage;
use bevy::shader::ShaderRef;

use wgpu_types::VertexFormat;

#[derive(Debug, Clone, Copy, Reflect, PartialEq, Eq, Hash)]
pub enum TerrainDebugType {
    Color,
    Normal,
}

pub const BIOME_VERTEX_ATTRIBUTE: MeshVertexAttribute =
    MeshVertexAttribute::new("biome", 100, VertexFormat::Uint32);

// 根据 terrain chunk 的 material 的类型来决定使用哪种地形材质。

/// 6 种 biome 的 PBR 基础颜色（Phase 3 spec）
const BIOME_COLORS: [BiomeColorUniform; 6] = [
    // Ocean: 深蓝灰
    BiomeColorUniform { biome: 0, base_color: Vec4::new(0.12, 0.18, 0.25, 1.0), _padding1: 0, _padding2: 0, _padding3: 0 },
    // Forest: 深绿
    BiomeColorUniform { biome: 1, base_color: Vec4::new(0.15, 0.35, 0.10, 1.0), _padding1: 0, _padding2: 0, _padding3: 0 },
    // Desert: 沙黄
    BiomeColorUniform { biome: 2, base_color: Vec4::new(0.76, 0.65, 0.35, 1.0), _padding1: 0, _padding2: 0, _padding3: 0 },
    // Plains: 草绿
    BiomeColorUniform { biome: 3, base_color: Vec4::new(0.25, 0.55, 0.15, 1.0), _padding1: 0, _padding2: 0, _padding3: 0 },
    // Mountains: 灰白
    BiomeColorUniform { biome: 4, base_color: Vec4::new(0.55, 0.55, 0.58, 1.0), _padding1: 0, _padding2: 0, _padding3: 0 },
    // Swamp: 暗绿棕
    BiomeColorUniform { biome: 5, base_color: Vec4::new(0.20, 0.28, 0.12, 1.0), _padding1: 0, _padding2: 0, _padding3: 0 },
];
// 如果是过渡部分，则使用过渡材质。

#[derive(Clone, Default, ShaderType)]
pub struct BiomeColorUniform {
    pub biome: u32,
    pub _padding1: u32,
    pub _padding2: u32,
    pub _padding3: u32,
    pub base_color: Vec4,
}

#[derive(Clone, Default, ShaderType)]
pub struct TerrainMaterialUniform {
    pub lod: u32,
    pub roughness: f32,
    pub metallic: f32,
    pub flags: u32,
    pub reflectance: f32,
    pub attenuation_distance: f32,
    pub attenuation_color: Vec4,
    pub biome_colors: [BiomeColorUniform; 6],
}

impl AsBindGroupShaderType<TerrainMaterialUniform> for TerrainMaterial {
    fn as_bind_group_shader_type(&self, images: &RenderAssets<GpuImage>) -> TerrainMaterialUniform {
        let mut flags = StandardMaterialFlags::NONE;
        if self.base_color_texture.is_some() {
            flags |= StandardMaterialFlags::BASE_COLOR_TEXTURE;
        }
        if self.metallic_roughness_texture.is_some() {
            flags |= StandardMaterialFlags::METALLIC_ROUGHNESS_TEXTURE;
        }
        if self.occlusion_texture.is_some() {
            flags |= StandardMaterialFlags::OCCLUSION_TEXTURE;
        }
        if self.double_sided {
            flags |= StandardMaterialFlags::DOUBLE_SIDED;
        }
        if self.unlit {
            flags |= StandardMaterialFlags::UNLIT;
        }
        if self.fog_enabled {
            flags |= StandardMaterialFlags::FOG_ENABLED;
        }

        let has_normal_map = self.normal_map_texture.is_some();
        if has_normal_map {
            let normal_map_id = self
                .normal_map_texture
                .as_ref()
                .map(Handle::id)
                .expect("Normal map texture handle missing");
            if let Some(texture) = images.get(normal_map_id) {
                match texture.texture_format {
                    TextureFormat::Rg8Unorm
                    | TextureFormat::Rg16Unorm
                    | TextureFormat::Bc5RgUnorm
                    | TextureFormat::EacRg11Unorm => {
                        flags |= StandardMaterialFlags::TWO_COMPONENT_NORMAL_MAP;
                    }
                    _ => {}
                }
            }
        }

        if self.attenuation_distance.is_finite() {
            flags |= StandardMaterialFlags::ATTENUATION_ENABLED;
        }

        TerrainMaterialUniform {
            lod: self.lod as u32,
            roughness: self.perceptual_roughness,
            metallic: self.metallic,
            flags: flags.bits(),
            reflectance: self.reflectance,
            attenuation_distance: self.attenuation_distance,
            attenuation_color: LinearRgba::from(self.attenuation_color)
                .to_f32_array()
                .into(),
            biome_colors: BIOME_COLORS,
        }
    }
}

#[derive(AsBindGroup, Clone, Asset, TypePath)]
#[bind_group_data(TerrainMaterialKey)]
#[uniform(0, TerrainMaterialUniform)]
pub struct TerrainMaterial {
    pub lod: u8,
    pub debug_type: Option<TerrainDebugType>,

    pub metallic: f32,
    pub perceptual_roughness: f32,

    // biome 0
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    pub base_color_texture: Option<Handle<Image>>,

    #[texture(3, dimension = "2d_array")]
    #[sampler(4)]
    pub normal_map_texture: Option<Handle<Image>>,

    // metallic is b channel, roughness is g channel
    #[texture(5, dimension = "2d_array")]
    #[sampler(6)]
    pub metallic_roughness_texture: Option<Handle<Image>>,

    // r channel
    #[texture(7, dimension = "2d_array")]
    #[sampler(8)]
    pub occlusion_texture: Option<Handle<Image>>,

    // biome 1
    #[texture(9, dimension = "2d_array")]
    pub base_color_texture_1: Option<Handle<Image>>,

    #[texture(10, dimension = "2d_array")]
    pub normal_map_texture_1: Option<Handle<Image>>,

    // metallic is b channel, roughness is g channel
    #[texture(11, dimension = "2d_array")]
    pub metallic_roughness_texture_1: Option<Handle<Image>>,

    // r channel
    #[texture(12, dimension = "2d_array")]
    pub occlusion_texture_2: Option<Handle<Image>>,

    pub double_sided: bool,
    pub cull_mode: Option<Face>,

    pub unlit: bool,

    pub fog_enabled: bool,

    pub reflectance: f32,

    pub attenuation_distance: f32,
    pub attenuation_color: Color,
}

impl Default for TerrainMaterial {
    fn default() -> Self {
        TerrainMaterial {
            lod: 0,
            debug_type: None,
            base_color_texture: None,
            normal_map_texture: None,
            metallic: 0.0,
            perceptual_roughness: 0.5,
            metallic_roughness_texture: None,
            occlusion_texture: None,
            double_sided: false,
            cull_mode: None,
            unlit: false,
            fog_enabled: true,
            reflectance: 0.5,
            attenuation_distance: f32::INFINITY,
            attenuation_color: Color::WHITE,
            base_color_texture_1: None,
            normal_map_texture_1: None,
            metallic_roughness_texture_1: None,
            occlusion_texture_2: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TerrainMaterialKey {
    pub debug_type: Option<TerrainDebugType>,
    pub cull_mode: Option<Face>,
    pub normal_map: bool,
}

impl From<&TerrainMaterial> for TerrainMaterialKey {
    fn from(value: &TerrainMaterial) -> Self {
        TerrainMaterialKey {
            debug_type: value.debug_type,
            cull_mode: value.cull_mode,
            normal_map: value.normal_map_texture.is_some(),
        }
    }
}

impl Material for TerrainMaterial {
    fn vertex_shader() -> ShaderRef {
        "shaders/terrain/render/terrain_vertex.wgsl".into()
    }

    fn prepass_vertex_shader() -> ShaderRef {
        "shaders/terrain/render/terrain_vertex.wgsl".into()
    }

    fn fragment_shader() -> ShaderRef {
        "shaders/terrain/render/terrain_fragment.wgsl".into()
    }

    fn prepass_fragment_shader() -> ShaderRef {
        "shaders/terrain/render/terrain_fragment.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = key.bind_group_data.cull_mode;

        const PREPASS: MeshPipelineKey = MeshPipelineKey::DEPTH_PREPASS
            .union(MeshPipelineKey::NORMAL_PREPASS)
            .union(MeshPipelineKey::MOTION_VECTOR_PREPASS);
        let is_prepass = !(key.mesh_key & PREPASS).is_empty();
        if is_prepass.not() {
            let fragment = descriptor
                .fragment
                .as_mut()
                .expect("Fragment descriptor missing");
            if let Some(debug_type) = key.bind_group_data.debug_type {
                match debug_type {
                    TerrainDebugType::Color => {
                        fragment.shader_defs.push("COLOR_DEBUG".into());
                    }
                    TerrainDebugType::Normal => {
                        fragment.shader_defs.push("NORMAL_DEBUG".into());
                    }
                }
            }

            if key.bind_group_data.normal_map {
                fragment
                    .shader_defs
                    .push("STANDARD_MATERIAL_NORMAL_MAP".into());
            }
        }

        let _vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            BIOME_VERTEX_ATTRIBUTE.at_shader_location(2),
        ])?;
        Ok(())
    }
}
