use std::{io::Write, path::Path};

use bevy::prelude::info;
use nalgebra::Vector3;

#[derive(Clone, Debug)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub normals: Vector3<f32>,
}

impl Vertex {
    pub fn new() -> Self {
        Self {
            position: Vector3::new(0.0, 0.0, 0.0),
            normals: Vector3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn new_with_position_and_normals(position: &Vector3<f32>, normals: &Vector3<f32>) -> Self {
        Self {
            position: position.clone(),
            normals: normals.clone(),
        }
    }
}

impl Vertex {
    pub fn set_position(&mut self, position: &Vector3<f32>) {
        self.position = position.clone();
    }

    pub fn set_normals(&mut self, normals: &Vector3<f32>) {
        self.normals = normals.clone();
    }

    pub fn get_position(&self) -> &Vector3<f32> {
        &self.position
    }

    pub fn get_normals(&self) -> &Vector3<f32> {
        &self.normals
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl Mesh {
    pub fn set_vertices(&mut self, vertices: Vec<Vertex>) {
        self.vertices = vertices;
    }

    pub fn set_indices(&mut self, indices: Vec<u32>) {
        self.indices = indices;
    }

    pub fn get_vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn get_indices(&self) -> &Vec<u32> {
        &self.indices
    }

    pub fn get_vertices_mut(&mut self) -> &mut Vec<Vertex> {
        &mut self.vertices
    }

    pub fn get_indices_mut(&mut self) -> &mut Vec<u32> {
        &mut self.indices
    }
}

impl Mesh {
    pub fn export_obj(&self, path: &Path) {
        info!("export_obj");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = std::fs::File::create(path).unwrap();

        for vertex in &self.vertices {
            let position = vertex.get_position();
            let normals = vertex.get_normals();

            let line = format!(
                "v {} {} {}\nvn {} {} {}\n",
                position.x, position.y, position.z, normals.x, normals.y, normals.z
            );

            file.write_all(line.as_bytes()).unwrap();
        }

        for i in (0..self.indices.len()).step_by(3) {
            let line = format!(
                "f {} {} {}\n",
                self.indices[i] + 1,
                self.indices[i + 1] + 1,
                self.indices[i + 2] + 1
            );

            file.write_all(line.as_bytes()).unwrap();
        }
    }
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
