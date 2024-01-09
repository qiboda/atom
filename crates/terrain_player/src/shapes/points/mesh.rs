
use bevy::{
    math::Vec2,
    prelude::{Color, Mesh, Vec3},
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};

#[derive(Default)]
pub struct PointsMesh {
    pub vertices: Vec<Vec3>,
    pub uv: Vec<Vec2>,
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

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        if let Some(color) = m.colors {
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_COLOR,
                color
                    .iter()
                    .flat_map(|c| {
                        let arr = c.as_rgba_f32();
                        [arr, arr, arr, arr]
                    })
                    .collect::<Vec<[f32; 4]>>(),
            );
        }
        mesh.set_indices(Some(indices));
        mesh
    }
}

impl PointsMesh {
    pub fn get_last_index(mesh: &Mesh) -> Option<usize> {
        if let Some(VertexAttributeValues::Float32x3(position)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            Some(position.len() / 4 - 1)
        } else {
            None
        }
    }

    pub fn add_point(mesh: &mut Mesh, point: &[f32; 3]) {
        if let Some(VertexAttributeValues::Float32x3(position)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            let idx = position.len() as u32;

            // info!("position len: {}", idx);

            (0..4).for_each(|_| position.push(point.clone()));

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

    pub fn remove_point_at_index(mesh: &mut Mesh, index: usize) {
        if let Some(VertexAttributeValues::Float32x3(position)) =
            mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION)
        {
            let index = index * 4;
            (0..4).for_each(|_| {
                position.remove(index);
            });
        }

        if let Some(VertexAttributeValues::Float32x2(uv)) = mesh.attribute_mut(Mesh::ATTRIBUTE_UV_0)
        {
            (0..4).for_each(|_| {
                uv.remove(index * 4);
            });
        }

        if let Some(Indices::U32(indices)) = mesh.indices_mut() {
            let idx = index * 6;
            (0..6).for_each(|_| {
                indices.remove(idx);
            });
        }
    }
}
