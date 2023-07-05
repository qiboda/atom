// use std::{io::Write, path::Path};

use bevy::{
    prelude::{Component, UVec3, Vec3},
    utils::HashMap,
};
use nalgebra::Vector3;

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

impl MeshCache {
    // pub fn export_obj(&self, path: &Path) {
    //     info!("export_obj");
    //     std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    //     let mut file = std::fs::File::create(path).unwrap();
    //
    //     for vertex in &self.vertices {
    //         let position = vertex.get_position();
    //         let normals = vertex.get_normals();
    //
    //         let line = format!(
    //             "v {} {} {}\nvn {} {} {}\n",
    //             position.x, position.y, position.z, normals.x, normals.y, normals.z
    //         );
    //
    //         file.write_all(line.as_bytes()).unwrap();
    //     }
    //
    //     for i in (0..self.indices.len()).step_by(3) {
    //         let line = format!(
    //             "f {} {} {}\n",
    //             self.indices[i] + 1,
    //             self.indices[i + 1] + 1,
    //             self.indices[i + 2] + 1
    //         );
    //
    //         file.write_all(line.as_bytes()).unwrap();
    //     }
    // }
}

#[cfg(test)]
mod test {
    use std::io::Write;

    #[test]
    fn test_write_files() {
        std::fs::create_dir_all("output/");
        let mut file = std::fs::File::create("output/abc.txt").unwrap();

        file.write_all("abcd".as_bytes());
        file.write_all("ebdadf".as_bytes());

        file.flush();
    }
}
