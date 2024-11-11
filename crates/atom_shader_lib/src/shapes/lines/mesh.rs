use bevy::{
    prelude::*,
    render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};

#[derive(Debug, Clone, Default)]
pub struct LineMesh {
    pub vertices: Vec<Vec3>,
    pub colors: Vec<Color>,
}

impl From<LineMesh> for Mesh {
    fn from(line_mesh: LineMesh) -> Self {
        debug_assert!(line_mesh.vertices.len() % 2 == 0);
        debug_assert!(line_mesh.colors.len() == line_mesh.vertices.len());

        let mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::MAIN_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, line_mesh.vertices)
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_COLOR,
                line_mesh
                    .colors
                    .iter()
                    .map(|c| c.to_linear().to_f32_array())
                    .collect::<Vec<[f32; 4]>>(),
            );

        mesh
    }
}
