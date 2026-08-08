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
