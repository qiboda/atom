use bevy::{prelude::*, reflect::TypeUuid, render::render_resource::AsBindGroup};

#[derive(AsBindGroup, TypeUuid, Clone, Default)]
#[uuid = "155a821f-9832-48e4-b460-0e8805cbbce5"]
pub struct CoolMaterial {
    #[uniform(0)]
    pub(crate) color: Color,

    #[uniform(1)]
    pub(crate) normal: Vec3,

    #[texture(2)]
    #[sampler(3)]
    pub(crate) color_texture: Handle<Image>,
}

impl Material for CoolMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        bevy::render::render_resource::ShaderRef::Default
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shader/cool_mat.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn depth_bias(&self) -> f32 {
        0.0
    }

    fn prepass_vertex_shader() -> bevy::render::render_resource::ShaderRef {
        bevy::render::render_resource::ShaderRef::Default
    }

    fn prepass_fragment_shader() -> bevy::render::render_resource::ShaderRef {
        bevy::render::render_resource::ShaderRef::Default
    }
}
