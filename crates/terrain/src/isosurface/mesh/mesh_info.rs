use std::ops::Not;

use avian3d::collision::Collider;
use bevy::{
    prelude::{debug, Component, Mesh, Vec3},
    reflect::Reflect,
    render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};

#[derive(Debug, Clone, Component, Default, Reflect)]
pub struct MeshInfo {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub indices: Vec<u32>,
}

impl MeshInfo {
    pub fn is_empty(&self) -> bool {
        self.positions.is_empty() || self.indices.is_empty()
    }

    fn check(&self) {
        debug_assert!(self.positions.is_empty().not());
        debug_assert!(self.positions.len() == self.normals.len());
        debug_assert!(self.indices.len() % 3 == 0);
    }

    pub fn get_vertex_positions(&self) -> &Vec<Vec3> {
        &self.positions
    }

    pub fn set_vertex_positions(&mut self, positions: Vec<Vec3>) {
        self.positions = positions;
    }

    pub fn get_vertex_normals(&self) -> &Vec<Vec3> {
        &self.normals
    }

    pub fn set_vertex_normals(&mut self, normals: Vec<Vec3>) {
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

impl From<&MeshInfo> for Mesh {
    fn from(mesh_cache: &MeshInfo) -> Self {
        mesh_cache.check();
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            mesh_cache.get_vertex_positions().clone(),
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            mesh_cache.get_vertex_normals().clone(),
        );
        debug!("mesh cache from: {:?}", mesh_cache.get_indices());
        mesh.insert_indices(bevy::render::mesh::Indices::U32(
            mesh_cache.get_indices().clone(),
        ));

        mesh
    }
}

impl From<&MeshInfo> for Collider {
    fn from(mesh_info: &MeshInfo) -> Self {
        mesh_info.check();

        let mut indices: Vec<[u32; 3]> = Vec::with_capacity(mesh_info.get_indices().len() / 3);
        for index in mesh_info.get_indices().chunks(3) {
            indices.push(index.try_into().unwrap());
        }

        Collider::trimesh(mesh_info.get_vertex_positions().clone(), indices)
    }
}
