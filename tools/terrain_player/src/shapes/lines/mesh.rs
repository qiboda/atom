use bevy::{prelude::*, render::render_resource::PrimitiveTopology};

#[derive(Debug, Clone, Default)]
struct LineMesh {
    pub vertices: Vec<Vec3>,
    pub colors: Option<Vec<Color>>,
}

impl From<LineMesh> for Mesh {

    fn from(line_mesh: LineMesh) -> Self {
        debug_assert!(line_mesh.vertices.len() % 2 == 0);
        debug_assert!(line_mesh.colors.is_none() || line_mesh.colors.as_ref().unwrap().len() == line_mesh.vertices.len());

        let mut mesh = Mesh::new(PrimitiveTopology::LineList)
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, line_mesh.vertices);

        if let Some(colors) = line_mesh.colors {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors.iter().map(|c| c.as_rgba_f32()).collect::<Vec<[f32; 4]>>());
        }
        mesh
    }
}