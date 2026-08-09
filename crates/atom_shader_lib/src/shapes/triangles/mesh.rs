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

    #[test]
    fn build_mesh_with_vertices_and_indices() {
        let mesh = TrianglesMesh::build_mesh(
            Some(vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]),
            Some(vec![0, 1, 2]),
        );
        assert_eq!(mesh.primitive_topology(), PrimitiveTopology::TriangleList);
        assert_eq!(
            positions(&mesh),
            [[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]].as_slice()
        );
        match mesh.indices() {
            Some(Indices::U32(idx)) => assert_eq!(idx.as_slice(), [0, 1, 2].as_slice()),
            _ => panic!("expected U32 indices"),
        }
    }

    #[test]
    fn build_mesh_with_none_is_empty() {
        let mesh = TrianglesMesh::build_mesh(None, None);
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_none());
        assert!(mesh.indices().is_none());
    }

    #[test]
    fn get_indices_len_returns_count() {
        let mesh = TrianglesMesh::build_mesh(None, Some(vec![0, 1, 2, 3, 4, 5]));
        assert_eq!(TrianglesMesh::get_indices_len(&mesh), Some(6));
    }

    #[test]
    fn get_indices_len_without_indices_returns_none() {
        let mesh = TrianglesMesh::build_mesh(Some(vec![[0., 0., 0.]]), None);
        assert_eq!(TrianglesMesh::get_indices_len(&mesh), None);
    }

    #[test]
    fn add_all_vertices_inserts_when_missing() {
        let mut mesh = TrianglesMesh::build_mesh(None, None);
        let vertices = vec![[1., 2., 3.], [4., 5., 6.]];
        TrianglesMesh::add_all_vertices(&mut mesh, &vertices);
        assert_eq!(positions(&mesh), vertices.as_slice());
    }

    #[test]
    fn add_all_vertices_replaces_when_empty() {
        let mut mesh = TrianglesMesh::build_mesh(Some(vec![]), None);
        let vertices = vec![[0., 0., 0.]];
        TrianglesMesh::add_all_vertices(&mut mesh, &vertices);
        assert_eq!(positions(&mesh), vertices.as_slice());
    }

    #[test]
    fn add_all_vertices_skips_when_non_empty() {
        let mut mesh = TrianglesMesh::build_mesh(Some(vec![[1., 1., 1.]]), None);
        TrianglesMesh::add_all_vertices(&mut mesh, &vec![[9., 9., 9.]]);
        assert_eq!(positions(&mesh), [[1., 1., 1.]].as_slice());
    }

    #[test]
    fn add_triangle_indices_appends() {
        let mut mesh = TrianglesMesh::build_mesh(
            Some(vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]),
            Some(vec![]),
        );
        TrianglesMesh::add_triangle_indices(&mut mesh, &[0, 1, 2]);
        assert_eq!(TrianglesMesh::get_indices_len(&mesh), Some(3));
        TrianglesMesh::add_triangle_indices(&mut mesh, &[0, 1, 2]);
        assert_eq!(TrianglesMesh::get_indices_len(&mesh), Some(6));
    }

    #[test]
    #[should_panic]
    fn add_triangle_indices_out_of_range_panics() {
        let mut mesh =
            TrianglesMesh::build_mesh(Some(vec![[0., 0., 0.], [1., 0., 0.]]), Some(vec![]));
        TrianglesMesh::add_triangle_indices(&mut mesh, &[0, 1, 5]);
    }

    #[test]
    #[should_panic]
    fn add_triangle_indices_without_position_panics() {
        let mut mesh = TrianglesMesh::build_mesh(None, Some(vec![]));
        TrianglesMesh::add_triangle_indices(&mut mesh, &[0, 1, 2]);
    }

    #[test]
    fn remove_last_triangle_indices_pops_three() {
        let mut mesh = TrianglesMesh::build_mesh(
            Some(vec![[0., 0., 0.], [1., 0., 0.], [0., 1., 0.]]),
            Some(vec![0, 1, 2, 0, 1, 2]),
        );
        TrianglesMesh::remove_last_triangle_indices(&mut mesh);
        assert_eq!(TrianglesMesh::get_indices_len(&mesh), Some(3));
        TrianglesMesh::remove_last_triangle_indices(&mut mesh);
        assert_eq!(TrianglesMesh::get_indices_len(&mesh), Some(0));
    }

    #[test]
    fn remove_last_triangle_indices_without_indices_is_noop() {
        let mut mesh = TrianglesMesh::build_mesh(Some(vec![[0., 0., 0.]]), None);
        TrianglesMesh::remove_last_triangle_indices(&mut mesh);
        assert!(mesh.indices().is_none());
    }
}
