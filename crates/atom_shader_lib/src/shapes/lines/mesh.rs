use bevy::{asset::RenderAssetUsages, prelude::*, render::render_resource::PrimitiveTopology};

/// 线段网格数据：顶点成对出现（每 2 个顶点构成一条线段）。
#[derive(Debug, Clone, Default)]
pub struct LineMesh {
    /// 顶点列表，长度必须为偶数（成对构成线段）。
    pub vertices: Vec<Vec3>,
    /// 每顶点颜色，长度须与 `vertices` 一致。
    pub colors: Vec<Color>,
}

impl From<LineMesh> for Mesh {
    fn from(line_mesh: LineMesh) -> Self {
        debug_assert!(line_mesh.vertices.len().is_multiple_of(2));
        debug_assert!(line_mesh.colors.len() == line_mesh.vertices.len());

        Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::MAIN_WORLD)
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, line_mesh.vertices)
            .with_inserted_attribute(
                Mesh::ATTRIBUTE_COLOR,
                line_mesh
                    .colors
                    .iter()
                    .map(|c| c.to_linear().to_f32_array())
                    .collect::<Vec<[f32; 4]>>(),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::mesh::VertexAttributeValues;

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(v)) => v,
            _ => panic!("expected Float32x3 position attribute"),
        }
    }

    fn colors(mesh: &Mesh) -> &[[f32; 4]] {
        match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            Some(VertexAttributeValues::Float32x4(v)) => v,
            _ => panic!("expected Float32x4 color attribute"),
        }
    }

    #[test]
    fn line_mesh_converts_to_line_list() {
        let mesh = Mesh::from(LineMesh {
            vertices: vec![Vec3::new(1., 2., 3.), Vec3::new(4., 5., 6.)],
            colors: vec![Color::WHITE; 2],
        });
        assert_eq!(mesh.primitive_topology(), PrimitiveTopology::LineList);
        assert_eq!(
            positions(&mesh),
            vec![[1., 2., 3.], [4., 5., 6.]].as_slice()
        );
        assert_eq!(colors(&mesh), &[[1., 1., 1., 1.]; 2]);
    }

    #[test]
    fn line_mesh_empty_is_empty_mesh() {
        let mesh = Mesh::from(LineMesh::default());
        assert!(positions(&mesh).is_empty());
        assert!(colors(&mesh).is_empty());
    }

    #[test]
    fn line_mesh_linearizes_vertex_colors() {
        let mesh = Mesh::from(LineMesh {
            vertices: vec![Vec3::ZERO; 2],
            colors: vec![Color::BLACK, Color::WHITE],
        });
        assert_eq!(&colors(&mesh)[0], &[0., 0., 0., 1.]);
        assert_eq!(&colors(&mesh)[1], &[1., 1., 1., 1.]);
    }
}
