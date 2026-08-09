use bevy::{
    asset::RenderAssetUsages,
    color::ColorToComponents,
    math::Vec2,
    mesh::{Indices, VertexAttributeValues},
    prelude::{Color, Mesh, Vec3},
    render::render_resource::PrimitiveTopology,
};

/// 点精灵网格数据：每个点展开为一个四边形面片（4 个顶点、6 个索引）。
#[derive(Default)]
pub struct PointsMesh {
    /// 点位置列表。
    pub vertices: Vec<Vec3>,
    /// 每点 UV 坐标（与 `vertices` 一一对应）。
    pub uv: Vec<Vec2>,
    /// 可选顶点颜色（为 `None` 时不写入颜色属性）。
    pub colors: Option<Vec<Color>>,
}

impl From<PointsMesh> for Mesh {
    fn from(m: PointsMesh) -> Self {
        let vertices: Vec<[f32; 3]> = m
            .vertices
            .iter()
            .flat_map(|p| {
                let arr = p.to_array();
                [arr, arr, arr, arr]
            })
            .collect();

        let uv_set = [[0.4, 0.4], [0.5, 0.4], [0.5, 0.5], [0.4, 0.5]];
        let uvs: Vec<[f32; 2]> = m.vertices.iter().flat_map(|_| uv_set).collect();

        let indices = Indices::U32(
            m.vertices
                .iter()
                .enumerate()
                .flat_map(|(i, _)| {
                    let idx = (i * 4) as u32;
                    [idx, idx + 1, idx + 3, idx + 2, idx + 3, idx + 1]
                })
                .collect(),
        );

        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD,
        );
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        if let Some(color) = m.colors {
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_COLOR,
                color
                    .iter()
                    .flat_map(|c| {
                        let arr = c.to_linear().to_f32_array();
                        [arr, arr, arr, arr]
                    })
                    .collect::<Vec<[f32; 4]>>(),
            );
        }
        mesh.insert_indices(indices);
        mesh
    }
}

impl PointsMesh {
    /// 返回网格中最后一个点的索引（按每点 4 顶点折算）。
    ///
    /// 网格须含 `ATTRIBUTE_POSITION` 且顶点数为 4 的倍数，否则返回 `None`。
    pub fn get_last_index(mesh: &Mesh) -> Option<usize> {
        if let Some(VertexAttributeValues::Float32x3(position)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            Some(position.len() / 4 - 1)
        } else {
            None
        }
    }

    /// 向已有网格末尾追加一个点（4 个顶点 + UV + 6 个索引）。
    pub fn add_point(mesh: &mut Mesh, point: &[f32; 3]) {
        if let Some(VertexAttributeValues::Float32x3(position)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            let idx = position.len() as u32;

            // info!("position len: {}", idx);

            (0..4).for_each(|_| position.push(*point));

            if let Some(VertexAttributeValues::Float32x2(uv)) =
                mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
            {
                let uv_set = [[0., 0.], [1., 0.], [1., 1.], [0., 1.]];
                uv.append(&mut uv_set.into_iter().collect::<Vec<[f32; 2]>>());
            }

            if let Some(Indices::U32(indices)) = mesh.indices_mut() {
                indices.append(
                    &mut [idx, idx + 1, idx + 3, idx + 2, idx + 3, idx + 1]
                        .into_iter()
                        .collect::<Vec<u32>>(),
                );
            }
        }
    }

    /// 从网格末尾移除一个点（当前实现忽略 `index`，始终弹出最后一组顶点/UV/索引）。
    pub fn remove_point_at_index(mesh: &mut Mesh, _index: usize) {
        if let Some(VertexAttributeValues::Float32x3(position)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            (0..4).for_each(|_| {
                position.pop();
            });
        }

        if let Some(VertexAttributeValues::Float32x2(uv)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            (0..4).for_each(|_| {
                uv.pop();
            });
        }

        if let Some(Indices::U32(indices)) = mesh.indices_mut() {
            (0..6).for_each(|_| {
                indices.pop();
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UV_SET: [[f32; 2]; 4] = [[0.4, 0.4], [0.5, 0.4], [0.5, 0.5], [0.4, 0.5]];

    fn positions(mesh: &Mesh) -> &[[f32; 3]] {
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(v)) => v,
            _ => panic!("expected Float32x3 position attribute"),
        }
    }

    fn uvs(mesh: &Mesh) -> &[[f32; 2]] {
        match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
            Some(VertexAttributeValues::Float32x2(v)) => v,
            _ => panic!("expected Float32x2 uv attribute"),
        }
    }

    fn indices(mesh: &Mesh) -> &[u32] {
        match mesh.indices() {
            Some(Indices::U32(v)) => v,
            _ => panic!("expected U32 indices"),
        }
    }

    #[test]
    fn mesh_from_points_builds_quad_per_point() {
        let mesh = Mesh::from(PointsMesh {
            vertices: vec![Vec3::ZERO, Vec3::X, Vec3::Y],
            uv: vec![Vec2::ZERO; 3],
            colors: None,
        });
        assert_eq!(mesh.primitive_topology(), PrimitiveTopology::TriangleList);
        // 每个点展开为 4 顶点 + 6 索引。
        assert_eq!(positions(&mesh).len(), 12);
        assert_eq!(uvs(&mesh).len(), 12);
        assert_eq!(indices(&mesh).len(), 18);
    }

    #[test]
    fn mesh_from_points_repeats_position_four_times() {
        let mesh = Mesh::from(PointsMesh {
            vertices: vec![Vec3::new(1., 2., 3.), Vec3::new(4., 5., 6.)],
            uv: Vec::new(),
            colors: None,
        });
        assert_eq!(
            positions(&mesh),
            [vec![[1., 2., 3.]; 4], vec![[4., 5., 6.]; 4]]
                .concat()
                .as_slice()
        );
    }

    #[test]
    fn mesh_from_points_uses_fixed_uv_set() {
        let mesh = Mesh::from(PointsMesh {
            vertices: vec![Vec3::ZERO; 2],
            uv: Vec::new(),
            colors: None,
        });
        let expected: Vec<[f32; 2]> = [UV_SET, UV_SET].concat();
        assert_eq!(uvs(&mesh), expected.as_slice());
    }

    #[test]
    fn mesh_from_points_without_colors_omits_color_attribute() {
        let mesh = Mesh::from(PointsMesh {
            vertices: vec![Vec3::ZERO],
            uv: Vec::new(),
            colors: None,
        });
        assert!(mesh.attribute(Mesh::ATTRIBUTE_COLOR).is_none());
    }

    #[test]
    fn mesh_from_points_with_colors_repeats_four_times() {
        let mesh = Mesh::from(PointsMesh {
            vertices: vec![Vec3::ZERO; 2],
            uv: Vec::new(),
            colors: Some(vec![Color::WHITE, Color::BLACK]),
        });
        let colors = match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            Some(VertexAttributeValues::Float32x4(v)) => v,
            _ => panic!("expected Float32x4 color attribute"),
        };
        let white = Color::WHITE.to_linear().to_f32_array();
        let black = Color::BLACK.to_linear().to_f32_array();
        assert_eq!(&colors[0..4], &[white; 4]);
        assert_eq!(&colors[4..8], &[black; 4]);
    }

    #[test]
    fn mesh_from_points_empty_builds_empty_attributes() {
        let mesh = Mesh::from(PointsMesh::default());
        assert!(positions(&mesh).is_empty());
        assert!(uvs(&mesh).is_empty());
        assert!(indices(&mesh).is_empty());
    }

    #[test]
    fn get_last_index_returns_last_point_index() {
        let mesh = Mesh::from(PointsMesh {
            vertices: vec![Vec3::ZERO; 4],
            uv: Vec::new(),
            colors: None,
        });
        assert_eq!(PointsMesh::get_last_index(&mesh), Some(3));
    }

    #[test]
    fn get_last_index_without_position_returns_none() {
        let mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD,
        );
        assert_eq!(PointsMesh::get_last_index(&mesh), None);
    }

    #[test]
    fn add_point_appends_quad() {
        let mut mesh = Mesh::from(PointsMesh {
            vertices: vec![Vec3::new(1., 1., 1.)],
            uv: Vec::new(),
            colors: None,
        });
        PointsMesh::add_point(&mut mesh, &[2., 3., 4.]);
        assert_eq!(positions(&mesh).len(), 8);
        assert_eq!(uvs(&mesh).len(), 8);
        assert_eq!(indices(&mesh).len(), 12);
        assert_eq!(&positions(&mesh)[4..8], &vec![[2., 3., 4.]; 4]);
    }

    #[test]
    fn add_point_appends_fixed_uv_and_indices() {
        let mut mesh = Mesh::from(PointsMesh {
            vertices: vec![Vec3::ZERO],
            uv: Vec::new(),
            colors: None,
        });
        PointsMesh::add_point(&mut mesh, &[0., 0., 0.]);
        // 原有点 UV 保持不变，追加点使用 [0,0]..[0,1] 的满铺 UV。
        assert_eq!(&uvs(&mesh)[0..4], &UV_SET);
        assert_eq!(&uvs(&mesh)[4..8], &[[0., 0.], [1., 0.], [1., 1.], [0., 1.]]);
        // 原有点索引不变，追加点索引从旧顶点数开始。
        assert_eq!(&indices(&mesh)[0..6], &[0, 1, 3, 2, 3, 1]);
        assert_eq!(&indices(&mesh)[6..12], &[4, 5, 7, 6, 7, 5]);
    }

    #[test]
    fn add_point_without_attributes_is_noop() {
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD,
        );
        PointsMesh::add_point(&mut mesh, &[0., 0., 0.]);
        assert!(mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_none());
        assert!(mesh.indices().is_none());
    }

    #[test]
    fn remove_point_at_index_removes_last_quad() {
        let mut mesh = Mesh::from(PointsMesh {
            vertices: vec![Vec3::new(1., 2., 3.), Vec3::new(4., 5., 6.)],
            uv: Vec::new(),
            colors: None,
        });
        PointsMesh::remove_point_at_index(&mut mesh, 0);
        assert_eq!(positions(&mesh).len(), 4);
        assert_eq!(uvs(&mesh).len(), 4);
        assert_eq!(indices(&mesh).len(), 6);
        assert_eq!(&positions(&mesh)[0..4], &vec![[1., 2., 3.]; 4]);
        assert_eq!(indices(&mesh), &[0, 1, 3, 2, 3, 1]);
    }

    #[test]
    fn remove_point_at_index_empty_mesh_is_noop() {
        let mut mesh = Mesh::from(PointsMesh::default());
        PointsMesh::remove_point_at_index(&mut mesh, 0);
        assert!(positions(&mesh).is_empty());
        assert!(uvs(&mesh).is_empty());
        assert!(indices(&mesh).is_empty());
    }
}
