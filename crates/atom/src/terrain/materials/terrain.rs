use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::render_resource::AsBindGroup,
};

// StandardMaterial::default()

#[derive(AsBindGroup, TypeUuid, TypePath, Clone, Default)]
#[uuid = "fd00e067-1a19-47e7-ae15-e05450d68230"]
pub struct TerrainMaterial {
    #[uniform(0)]
    pub(crate) base_color: Color,

    #[texture(1)]
    #[sampler(2)]
    pub(crate) base_color_texture: Option<Handle<Image>>,

    #[texture(3)]
    #[sampler(4)]
    pub normal_map_texture: Option<Handle<Image>>,

    #[texture(5)]
    #[sampler(6)]
    pub metallic_texture: Option<Handle<Image>>,

    #[texture(7)]
    #[sampler(8)]
    pub roughness_texture: Option<Handle<Image>>,

    #[texture(9)]
    #[sampler(10)]
    pub occlusion_texture: Option<Handle<Image>>,
}

impl Material for TerrainMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        bevy::render::render_resource::ShaderRef::Default
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        // bevy::render::render_resource::ShaderRef::Default
        "shader/terrain/terrain_mat.wgsl".into()
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
