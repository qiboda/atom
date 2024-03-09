use bevy::{
    prelude::Mesh,
    render::{mesh::Indices, render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};

#[derive(Default)]
pub struct TrianglesMesh;

impl TrianglesMesh {
    pub fn build_mesh(vertices: Option<Vec<[f32; 3]>>, indices: Option<Vec<u32>>) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD);
        if let Some(vertices) = vertices {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        }
        if let Some(indices) = indices {
            mesh.insert_indices(Indices::U32(indices));
        }
        mesh
    }

    #[allow(dead_code)]
    pub fn get_indices_len(mesh: &Mesh) -> Option<usize> {
        if let Some(Indices::U32(indices)) = mesh.indices() {
            Some(indices.len())
        } else {
            None
        }
    }

    #[allow(clippy::ptr_arg)]
    pub fn add_all_vertices(mesh: &mut Mesh, vertices: &Vec<[f32; 3]>) {
        if mesh.attribute(Mesh::ATTRIBUTE_POSITION).is_none() {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
        } else if let Some(positions) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
            if positions.is_empty() {
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
            }
        }
    }

    pub fn add_triangle_indices(mesh: &mut Mesh, add_indices: &[u32; 3]) {
        assert!(mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .map(|attributes| {
                add_indices
                    .iter()
                    .all(|index| *index < attributes.len() as u32)
            })
            .unwrap());

        if let Some(Indices::U32(indices)) = mesh.indices_mut() {
            add_indices.iter().for_each(|index| {
                indices.push(*index);
            });
        }
    }

    pub fn remove_last_triangle_indices(mesh: &mut Mesh) {
        if let Some(Indices::U32(indices)) = mesh.indices_mut() {
            (0..3).for_each(|_| {
                indices.pop();
            });
        };
    }
}
