use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};

#[derive(AsBindGroup, Clone, Default, Asset, Reflect)]
pub struct TerrainDebugMaterial {
    #[uniform(0)]
    pub(crate) color: LinearRgba,
}

impl Material for TerrainDebugMaterial {
    fn fragment_shader() -> ShaderRef {
        "shader/terrain/terrain_debug_mat.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shader/terrain/terrain_debug_mat.wgsl".into()
    }

    fn specialize(
        pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayoutRef,
        key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {

        Ok(())
    }
}
