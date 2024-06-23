use bevy::{
    prelude::{debug, Component, Mesh, Vec3}, reflect::Reflect, render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology}
};
// use bevy_xpbd_3d::{
//     parry::{
//         math::{Point, Real},
//         shape::SharedShape,
//     },
//     prelude::Collider,
// };

#[derive(Debug, Clone, Copy, Component, Default)]
pub struct TerrainChunkMesh;

#[derive(Debug, Clone, Component, Default, Reflect)]
pub struct MeshCache {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub indices: Vec<u32>,
}

impl MeshCache {
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty() || self.indices.is_empty()
    }

    fn check(&self) {
        debug_assert!(
            !self.positions.is_empty()
                && self.positions.len() == self.normals.len()
                && self.indices.len() % 3 == 0
        );
    }

    pub fn get_vertice_positions(&self) -> &Vec<Vec3> {
        &self.positions
    }

    pub fn set_vertice_positions(&mut self, positions: Vec<Vec3>) {
        self.positions = positions;
    }

    pub fn get_vertice_normals(&self) -> &Vec<Vec3> {
        &self.normals
    }

    pub fn set_vertice_normals(&mut self, normals: Vec<Vec3>) {
        self.normals = normals;
    }

    pub fn set_indices(&mut self, indices: Vec<u32>) {
        self.indices = indices;
    }

    pub fn get_indices(&self) -> &Vec<u32> {
        &self.indices
    }

    pub fn get_indices_mut(&mut self) -> &mut Vec<u32> {
        &mut self.indices
    }
}

impl From<&MeshCache> for Mesh {
    fn from(mesh_cache: &MeshCache) -> Self {
        mesh_cache.check();
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::MAIN_WORLD);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            mesh_cache.get_vertice_positions().clone(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            mesh_cache.get_vertice_normals().clone(),
        );
        debug!("mesh cache from: {:?}", mesh_cache.get_indices());
        mesh.insert_indices(bevy::render::mesh::Indices::U32(
            mesh_cache.get_indices().clone(),
        ));
        mesh
    }
}

// impl From<&MeshCache> for Collider {
//     fn from(mesh_cache: &MeshCache) -> Self {
//         mesh_cache.check();

//         let mut vertices: Vec<Point<Real>> =
//             Vec::with_capacity(mesh_cache.get_vertice_positions().len());
//         let mut indices: Vec<[u32; 3]> = Vec::with_capacity(mesh_cache.get_indices().len() / 3);

//         mesh_cache
//             .get_vertice_positions()
//             .iter()
//             .for_each(|vertex| {
//                 vertices.push(Point::from_slice(&[vertex.x, vertex.y, vertex.z]));
//             });

//         for index in mesh_cache.get_indices().chunks(3) {
//             indices.push(index.try_into().unwrap());
//         }

//         Collider::from(SharedShape::trimesh(vertices, indices))
//     }
// }
