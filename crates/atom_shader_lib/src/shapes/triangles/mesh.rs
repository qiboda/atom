use bevy::{
    asset::RenderAssetUsages, mesh::Indices, prelude::Mesh,
    render::render_resource::PrimitiveTopology,
};

/// 三角形网格构造器：提供 `TriangleList` 网格的创建与索引/顶点的增量修改。
#[derive(Default)]
pub struct TrianglesMesh;

impl TrianglesMesh {
    /// 从可选的顶点与索引数据构建三角形网格。
    pub fn build_mesh(vertices: Option<Vec<[f32; 3]>>, indices: Option<Vec<u32>>) -> Mesh {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD,
        );
        if let Some(vertices) = vertices {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        }
        if let Some(indices) = indices {
            mesh.insert_indices(Indices::U32(indices));
        }
        mesh
    }

    /// 返回网格当前索引数量；网格未含 `U32` 索引时返回 `None`。
    #[allow(dead_code)]
    pub fn get_indices_len(mesh: &Mesh) -> Option<usize> {
        if let Some(Indices::U32(indices)) = mesh.indices() {
            Some(indices.len())
        } else {
            None
        }
    }

    /// 将 `vertices` 写入网格的 `ATTRIBUTE_POSITION`（仅在属性缺失或为空时写入）。
    #[allow(clippy::ptr_arg)]
    pub fn add_all_vertices(mesh: &mut Mesh, vertices: &Vec<[f32; 3]>) {
        if mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_none() {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
        } else if let Some(positions) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
            && positions.is_empty()
        {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
        }
    }

    /// 追加一个三角形（3 个索引），并断言所有索引小于当前顶点数。
    pub fn add_triangle_indices(mesh: &mut Mesh, add_indices: &[u32; 3]) {
        assert!(
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
                .map(|attributes| {
                    add_indices
                        .iter()
                        .all(|index| *index < attributes.len() as u32)
                })
                .expect("add triangle indices should success")
        );

        if let Some(Indices::U32(indices)) = mesh.indices_mut() {
            add_indices.iter().for_each(|index| {
                indices.push(*index);
            });
        }
    }

    /// 移除最后一个三角形的 3 个索引。
    pub fn remove_last_triangle_indices(mesh: &mut Mesh) {
        if let Some(Indices::U32(indices)) = mesh.indices_mut() {
            (0..3).for_each(|_| {
                indices.pop();
            });
        };
    }
}
