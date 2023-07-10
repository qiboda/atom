use bevy::{
    prelude::{Component, Mesh, UVec3, Vec3},
    render::render_resource::PrimitiveTopology,
    utils::HashMap,
};

use super::vertex_index::VertexIndices;

#[derive(Debug, Clone, Default, Component)]
pub struct MeshCache {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub vertex_index_data: HashMap<UVec3, VertexIndices>,
}

impl MeshCache {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
            vertex_index_data: HashMap::default(),
        }
    }
}

impl MeshCache {
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

impl From<MeshCache> for Mesh {
    fn from(mesh_cache: MeshCache) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            mesh_cache.get_vertice_positions().clone(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            mesh_cache.get_vertice_normals().clone(),
        );
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(
            mesh_cache.get_indices().clone(),
        )));
        mesh
    }
}
