use bevy::color::LinearRgba;
use bevy::log::warn;
use bevy::mesh::PrimitiveTopology;
use bevy::render::render_resource::PolygonMode;
use bevy::shader::ShaderDefVal;
use bevy::{
    asset::Asset,
    pbr::{MAX_CASCADES_PER_LIGHT, MAX_DIRECTIONAL_LIGHTS},
    prelude::{AlphaMode, Material, Mesh},
    reflect::TypePath,
    render::render_resource::{AsBindGroup, ShaderType},
};

use super::plugin::POINT_SHADER_HANDLE;

/// 点精灵渲染的 uniform 设置，随绑定组一并上传 GPU。
#[derive(Debug, Clone, Copy, ShaderType)]
pub struct PointsShaderSettings {
    /// 点大小（世界空间尺寸）。
    pub point_size: f32,
    /// 不透明度。
    pub opacity: f32,
    /// 点基础颜色。
    pub color: LinearRgba,
}

impl Default for PointsShaderSettings {
    fn default() -> Self {
        Self {
            point_size: 0.01,
            opacity: 1.0,
            color: LinearRgba::WHITE,
        }
    }
}

/// 点精灵材质：以四边形面片模拟点，支持透视缩放、圆形裁剪与顶点颜色。
#[derive(AsBindGroup, Debug, Clone, Copy, TypePath, Asset)]
#[bind_group_data(PointsMaterialKey)]
pub struct PointsMaterial {
    /// 材质 uniform 设置（点大小、不透明度与颜色）。
    #[uniform(0)]
    pub settings: PointsShaderSettings,
    /// 深度偏移量，用于缓解与地表相交时的 z-fighting。
    pub depth_bias: f32,
    /// 混合模式。
    pub alpha_mode: AlphaMode,
    /// 是否启用顶点颜色（网格需带 `ATTRIBUTE_COLOR`）。
    pub use_vertex_color: bool,
    /// 是否按透视距离缩放点大小。
    pub perspective: bool,
    /// 是否将四边形裁剪为圆形。
    pub circle: bool,
}

impl Default for PointsMaterial {
    fn default() -> Self {
        Self {
            settings: PointsShaderSettings::default(),
            depth_bias: 0.,
            alpha_mode: Default::default(),
            use_vertex_color: false,
            perspective: false,
            circle: false,
        }
    }
}

/// 点精灵材质的管线特化键，用于区分不同的 shader 定义组合（顶点颜色 / 透视 / 圆形）。
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct PointsMaterialKey {
    use_vertex_color: bool,
    perspective: bool,
    circle: bool,
}

impl From<&PointsMaterial> for PointsMaterialKey {
    fn from(material: &PointsMaterial) -> Self {
        PointsMaterialKey {
            use_vertex_color: material.use_vertex_color,
            perspective: material.perspective,
            circle: material.circle,
        }
    }
}

impl Material for PointsMaterial {
    fn vertex_shader() -> bevy::shader::ShaderRef {
        POINT_SHADER_HANDLE.into()
    }

    fn fragment_shader() -> bevy::shader::ShaderRef {
        POINT_SHADER_HANDLE.into()
    }

    fn alpha_mode(&self) -> bevy::prelude::AlphaMode {
        self.alpha_mode
    }

    fn depth_bias(&self) -> f32 {
        self.depth_bias
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::mesh::MeshVertexBufferLayoutRef,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.cull_mode = None;
        descriptor.primitive.polygon_mode = PolygonMode::Fill;
        descriptor.primitive.topology = PrimitiveTopology::TriangleList;

        let mut shader_defs = vec![];
        let mut vertex_attributes = vec![
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
        ];

        // FIXME: To fix compilation errors in WGSL, the definitions of lights need to be resolved.
        shader_defs.push(ShaderDefVal::UInt(
            "MAX_DIRECTIONAL_LIGHTS".to_string(),
            MAX_DIRECTIONAL_LIGHTS as u32,
        ));
        shader_defs.push(ShaderDefVal::UInt(
            "MAX_CASCADES_PER_LIGHT".to_string(),
            MAX_CASCADES_PER_LIGHT as u32,
        ));

        if key.bind_group_data.use_vertex_color && layout.0.contains(Mesh::ATTRIBUTE_COLOR) {
            shader_defs.push(ShaderDefVal::from("VERTEX_COLORS"));
            vertex_attributes.push(Mesh::ATTRIBUTE_COLOR.at_shader_location(2));
        }
        if key.bind_group_data.perspective {
            shader_defs.push(ShaderDefVal::from("POINT_SIZE_PERSPECTIVE"));
            warn!("POINT_SIZE_PERSPECTIVE");
        }
        if key.bind_group_data.circle {
            shader_defs.push(ShaderDefVal::from("POINT_SHAPE_CIRCLE"));
        }

        let vertex_layout = layout.0.get_layout(&vertex_attributes)?;
        descriptor.vertex.buffers = vec![vertex_layout];
        descriptor.vertex.shader_defs.clone_from(&shader_defs);
        if let Some(fragment) = &mut descriptor.fragment {
            fragment.shader_defs = shader_defs;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::color::ColorToComponents;

    #[test]
    fn shader_settings_default() {
        let settings = PointsShaderSettings::default();
        assert!((settings.point_size - 0.01).abs() < 1e-6);
        assert!((settings.opacity - 1.0).abs() < 1e-6);
        assert_eq!(settings.color.to_f32_array(), [1., 1., 1., 1.]);
    }

    #[test]
    fn material_default() {
        let material = PointsMaterial::default();
        assert!((material.depth_bias - 0.).abs() < 1e-6);
        assert_eq!(material.alpha_mode, AlphaMode::Opaque);
        assert!(!material.use_vertex_color);
        assert!(!material.perspective);
        assert!(!material.circle);
        assert_eq!(
            material.settings.color.to_f32_array(),
            PointsShaderSettings::default().color.to_f32_array()
        );
    }

    #[test]
    fn key_from_material_maps_flags() {
        let enabled = PointsMaterial {
            settings: PointsShaderSettings::default(),
            depth_bias: 0.1,
            alpha_mode: AlphaMode::Blend,
            use_vertex_color: true,
            perspective: true,
            circle: true,
        };
        let key = PointsMaterialKey::from(&enabled);
        assert!(key.use_vertex_color);
        assert!(key.perspective);
        assert!(key.circle);

        let key = PointsMaterialKey::from(&PointsMaterial::default());
        assert!(!key.use_vertex_color);
        assert!(!key.perspective);
        assert!(!key.circle);
    }

    #[test]
    fn key_equality_and_hash() {
        let key_a = PointsMaterialKey::from(&PointsMaterial {
            use_vertex_color: true,
            perspective: false,
            circle: true,
            ..PointsMaterial::default()
        });
        let key_b = PointsMaterialKey::from(&PointsMaterial {
            use_vertex_color: true,
            perspective: false,
            circle: true,
            ..PointsMaterial::default()
        });
        assert_eq!(key_a.use_vertex_color, key_b.use_vertex_color);
        assert_eq!(key_a.perspective, key_b.perspective);
        assert_eq!(key_a.circle, key_b.circle);
        assert_eq!(hash_of(&key_a), hash_of(&key_b));

        let key_c = PointsMaterialKey::from(&PointsMaterial {
            use_vertex_color: false,
            perspective: false,
            circle: true,
            ..PointsMaterial::default()
        });
        assert_ne!(key_a.use_vertex_color, key_c.use_vertex_color);
        assert_eq!(key_a.circle, key_c.circle);
    }

    fn hash_of<T: std::hash::Hash>(value: &T) -> u64 {
        use std::hash::Hasher;
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }
}
