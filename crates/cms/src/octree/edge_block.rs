use nalgebra::Vector3;

use super::tables::Direction;

#[derive(Default, Clone, Debug)]
pub struct VertexIndices {
    empty: bool,
    vertex_index: Vector3<Option<usize>>,
}

impl VertexIndices {
    pub fn new() -> Self {
        VertexIndices {
            empty: true,
            vertex_index: Vector3::new(None, None, None),
        }
    }
}

impl VertexIndices {
    pub fn set_vertex_index(&mut self, x: usize, y: usize, z: usize) {
        self.vertex_index = Vector3::new(Some(x), Some(y), Some(z));
        self.empty = false;
    }

    pub fn set_dir_vertex_index(&mut self, dir: Direction, vertex_index: usize) {
        match dir {
            Direction::XAxis => self.vertex_index.x = Some(vertex_index),
            Direction::YAxis => self.vertex_index.y = Some(vertex_index),
            Direction::ZAxis => self.vertex_index.z = Some(vertex_index),
        }
        self.empty = false;
    }

    pub fn get_vertex_index(&self) -> &Vector3<Option<usize>> {
        &self.vertex_index
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn set_empty(&mut self) {
        self.empty = false;
        self.vertex_index = Vector3::new(None, None, None);
    }
}
