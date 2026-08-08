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
